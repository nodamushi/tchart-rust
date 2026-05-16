//! Font resolution and a [`FontMetrics`] adapter backed by `fontdue`.
//!
//! Resolution precedence (`docs/spec/cli.md` §フォント解決の優先順位):
//! 1. `--font <FILE>` argument.
//! 2. `TCHART_FONT` environment variable.
//! 3. OS scan via well-known font directories.
//!
//! Family resolution (`docs/spec/cli.md` §family解決):
//! - Linux: `fc-match -f '%{file}'` to obtain the font file for a given family.
//! - Unresolvable families fall back to the default font and emit a warning.
//!
//! Shared font cache (`docs/spec/cli.md` §並列ワーカと共有フォントキャッシュ):
//! - One `SharedFontCache` instance is shared across all workers in one CLI run.
//! - Family resolution happens at most once per family name (guarded by
//!   `OnceLock`-equivalent semantics inside `SharedFontCache`).
//! - Font file loading happens at most once per path (guarded by `OnceLock`
//!   per path entry).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use fontdue::Font;
use tchart_core::layout::FontMetrics;
use tchart_core::text::FontSpec;
use tchart_core::units::Px;

use crate::error::{CliError, FontError};

/// Candidate font paths searched in order when neither `--font` nor
/// `TCHART_FONT` is provided. Exposed at crate visibility so integration
/// tests can probe the same list rather than maintaining a duplicate.
pub const CANDIDATE_FONTS: &[&str] = &[
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    "/usr/share/fonts/liberation-sans/LiberationSans-Regular.ttf",
    "/Library/Fonts/Arial.ttf",
    "/System/Library/Fonts/Helvetica.ttc",
    "C:\\Windows\\Fonts\\arial.ttf",
];

/// CSS generic family names that must be assigned in `fontdb`.
const CSS_GENERICS: &[&str] = &["sans-serif", "serif", "monospace", "cursive", "fantasy"];

// ---------------------------------------------------------------------------
// ResolvedFont
// ---------------------------------------------------------------------------

/// A single loaded font: raw bytes for `fontdb` and a parsed `fontdue` face
/// for text-width measurement.
struct ResolvedFont {
    font: Font,
    font_bytes: Arc<Vec<u8>>,
}

impl ResolvedFont {
    fn load(path: &Path) -> Result<Self, CliError> {
        let raw = std::fs::read(path).map_err(|source| FontError::ReadFailed {
            path: path.to_path_buf(),
            source,
        })?;
        let font_bytes = Arc::new(raw);
        let font = Font::from_bytes(font_bytes.as_slice(), fontdue::FontSettings::default())
            .map_err(|message| FontError::ParseFailed(message.to_owned()))?;
        Ok(Self { font, font_bytes })
    }

    /// Raw bytes for `fontdb` registration.
    fn font_bytes(&self) -> &[u8] {
        &self.font_bytes
    }

    /// Measure the rendered advance width of `text` at size `size_px`.
    fn measure_text_width(&self, text: &str, size_px: f32) -> Px {
        let total: f32 = text
            .chars()
            .map(|character| self.font.metrics(character, size_px).advance_width)
            .sum();
        Px(total)
    }
}

// ---------------------------------------------------------------------------
// SharedFontCache
// ---------------------------------------------------------------------------

/// A resolved font entry in the shared cache, together with the fontdb family
/// name extracted from the file (needed to assign CSS generics in `fontdb`).
pub(crate) struct FontEntry {
    resolved: Arc<ResolvedFont>,
    family_name: String,
}

/// Process-wide font cache shared across all parallel workers.
///
/// Guarantees defined in `docs/spec/cli.md` §共有フォントキャッシュ:
/// - Each font file is loaded from disk at most once (per path).
/// - Each family name is resolved (via OS) at most once.
/// - A warning about a failed family is emitted at most once (per family name).
/// - Map key insertion uses a short write lock; the load itself is guarded by a
///   per-cell `OnceLock` so multiple workers can load different fonts in parallel
///   without holding any map-level lock during the I/O.
pub(crate) struct SharedFontCache {
    default_font: Arc<ResolvedFont>,
    default_family_name: String,

    /// family name → per-cell `OnceLock` holding the resolution outcome.
    ///
    /// The `OnceLock` ensures resolution runs exactly once even when multiple
    /// workers race on the same family.  A new cell (empty) is inserted under a
    /// short write lock; subsequent workers that miss the read lock but find the
    /// cell already present wait on `OnceLock::get_or_init` instead of
    /// attempting another OS lookup.
    resolution_cells: RwLock<HashMap<String, Arc<OnceLock<Option<PathBuf>>>>>,

    /// font path → per-cell `OnceLock` holding the loaded `FontEntry`.
    ///
    /// Same principle: cell insertion is a short write lock; the actual file
    /// read is done inside `OnceLock::get_or_init`, so two workers loading
    /// different paths do not block each other.
    font_cells: RwLock<HashMap<PathBuf, Arc<OnceLock<FontEntry>>>>,

    /// Families for which a warning has already been emitted.
    ///
    /// Protected by a `Mutex`; inserting a family name before printing the
    /// warning ensures at most one thread prints it.
    warned_families: Mutex<HashSet<String>>,
}

impl SharedFontCache {
    /// Create a cache seeded with the default font already loaded.
    pub(crate) fn new(default_path: &Path) -> Result<Self, CliError> {
        let default_font = Arc::new(ResolvedFont::load(default_path)?);
        let default_family_name = extract_first_family_name(&default_font)?;

        // Pre-register the default font path so it is never loaded twice.
        let path_cell: Arc<OnceLock<FontEntry>> = Arc::new(OnceLock::new());
        let default_font_clone = Arc::clone(&default_font);
        let default_name_clone = default_family_name.clone();
        path_cell
            .set(FontEntry {
                resolved: default_font_clone,
                family_name: default_name_clone,
            })
            .unwrap_or_else(|_| unreachable!("fresh OnceLock"));

        let mut font_cells = HashMap::new();
        font_cells.insert(default_path.to_path_buf(), path_cell);

        Ok(Self {
            default_font,
            default_family_name,
            resolution_cells: RwLock::new(HashMap::new()),
            font_cells: RwLock::new(font_cells),
            warned_families: Mutex::new(HashSet::new()),
        })
    }

    /// Resolve a `@font` CSV value (e.g. `"NoSuchFont, sans-serif"`) and return
    /// the CSS generic assignment (if any) and the resolved font entry reference.
    ///
    /// The returned `Arc<FontEntry>` is the font that was resolved for this
    /// document, enabling the worker to register only the fonts it actually needs.
    ///
    /// Tries comma-separated candidates left-to-right; uses the first that
    /// resolves successfully. If nothing resolves, falls back to the default and
    /// emits a warning to stderr (exactly once per family name per process run).
    pub(crate) fn resolve_family_csv(
        &self,
        family_csv: &str,
    ) -> (Option<(String, String)>, Arc<FontEntry>) {
        let unquoted = strip_font_quotes(family_csv);
        for candidate in unquoted
            .split(',')
            .map(str::trim)
            .filter(|candidate| !candidate.is_empty())
        {
            if let Some((fontdb_family_name, font_entry)) = self.try_resolve_one_family(candidate) {
                let generic_assignment = if CSS_GENERICS.contains(&candidate) {
                    Some((candidate.to_owned(), fontdb_family_name))
                } else {
                    None
                };
                return (generic_assignment, font_entry);
            }
        }
        // All candidates failed: warn once and fall back to the default.
        self.emit_warning_once(unquoted);
        // Record any generic in the CSV as mapping to the default family name.
        let generic_assignment = unquoted
            .split(',')
            .map(str::trim)
            .find(|candidate| CSS_GENERICS.contains(candidate))
            .map(|generic| (generic.to_owned(), self.default_family_name.clone()));
        let default_entry = self.default_font_entry();
        (generic_assignment, default_entry)
    }

    /// Build a `fontdb::Database` populated with the default font plus the
    /// fonts referenced by `resolved_entries`.
    pub(crate) fn build_fontdb_for_document(
        &self,
        resolved_entries: &[Arc<FontEntry>],
        generic_assignments: &HashMap<String, String>,
    ) -> fontdb::Database {
        let mut database = fontdb::Database::new();
        database.load_font_data(self.default_font.font_bytes().to_vec());
        for entry in resolved_entries {
            // Avoid registering the default font twice.
            if !Arc::ptr_eq(&entry.resolved, &self.default_font) {
                database.load_font_data(entry.resolved.font_bytes().to_vec());
            }
        }
        assign_css_generics(
            &mut database,
            generic_assignments,
            &self.default_family_name,
        );
        database
    }

    /// Return the default font as an `Arc<FontEntry>` for uniform handling.
    fn default_font_entry(&self) -> Arc<FontEntry> {
        Arc::new(FontEntry {
            resolved: Arc::clone(&self.default_font),
            family_name: self.default_family_name.clone(),
        })
    }

    /// Attempt to resolve exactly one family name (not CSV) from the OS.
    ///
    /// Returns the `fontdb` family name and font entry on success, `None` when
    /// the family is unknown or its font file cannot be loaded.
    fn try_resolve_one_family(&self, family: &str) -> Option<(String, Arc<FontEntry>)> {
        // Lock ordering: resolution_cells (read/write) → font_cells (write) → font_cells (read).
        // Acquiring resolution first, then font_cells, prevents deadlock because
        // no code path acquires them in reverse order.
        let resolution_cell = self.resolution_cell_for(family);
        let outcome = resolution_cell.get_or_init(|| {
            resolve_family_to_path(family).inspect(|path| {
                // Pre-register the font cell so we hold a write lock only briefly.
                self.ensure_font_cell_exists(path);
            })
        });
        let font_path = outcome.as_ref()?;
        let font_cell = self.font_cell_for(font_path)?;
        font_cell.get_or_init(|| self.load_font_entry(font_path));
        font_cell.get().map(|entry| {
            (
                entry.family_name.clone(),
                Arc::new(FontEntry {
                    resolved: Arc::clone(&entry.resolved),
                    family_name: entry.family_name.clone(),
                }),
            )
        })
    }

    /// Load a `FontEntry` for `path`, falling back to the default on error.
    ///
    /// Called at most once per path via `OnceLock::get_or_init`.
    fn load_font_entry(&self, font_path: &Path) -> FontEntry {
        match ResolvedFont::load(font_path) {
            Ok(resolved_font) => {
                let family_name = extract_first_family_name(&resolved_font).unwrap_or_default();
                FontEntry {
                    resolved: Arc::new(resolved_font),
                    family_name,
                }
            }
            Err(error) => {
                // Emit a load-failure warning and fill the cell with the
                // default so subsequent workers do not retry the load.
                eprintln!("tchart: warning: could not load font: {error}");
                FontEntry {
                    resolved: Arc::clone(&self.default_font),
                    family_name: self.default_family_name.clone(),
                }
            }
        }
    }

    /// Return (or insert) the per-family `OnceLock` cell under a read-first /
    /// write-on-miss strategy.
    fn resolution_cell_for(&self, family: &str) -> Arc<OnceLock<Option<PathBuf>>> {
        // Fast path: read lock.
        {
            let guard = self
                .resolution_cells
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(cell) = guard.get(family) {
                return Arc::clone(cell);
            }
        }
        // Slow path: write lock to insert.
        let mut guard = self
            .resolution_cells
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard
            .entry(family.to_owned())
            .or_insert_with(|| Arc::new(OnceLock::new()))
            .clone()
    }

    /// Return the per-path `OnceLock` cell, or `None` when the path has no
    /// registered cell (should not happen after `ensure_font_cell_exists`).
    fn font_cell_for(&self, path: &Path) -> Option<Arc<OnceLock<FontEntry>>> {
        let guard = self
            .font_cells
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.get(path).map(Arc::clone)
    }

    /// Insert an empty font cell for `path` if one does not already exist.
    ///
    /// Uses a single write lock with `entry().or_insert_with()` to avoid
    /// the TOCTOU window that a read-then-write pattern would introduce.
    fn ensure_font_cell_exists(&self, path: &Path) {
        let mut guard = self
            .font_cells
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard
            .entry(path.to_path_buf())
            .or_insert_with(|| Arc::new(OnceLock::new()));
    }

    /// Emit a "font family not found" warning to stderr at most once for
    /// `family_csv`.
    fn emit_warning_once(&self, family_csv: &str) {
        let mut guard = self
            .warned_families
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if guard.insert(family_csv.to_owned()) {
            eprintln!("tchart: warning: font family not found: {family_csv}; using default");
        }
    }

    /// Measure text width using the default font. Worker-specific family
    /// measurement is not available through the `FontMetrics` trait (no family
    /// name parameter); the default font approximation is acceptable for layout.
    fn measure_default(&self, text: &str, size_px: f32) -> Px {
        self.default_font.measure_text_width(text, size_px)
    }
}

// ---------------------------------------------------------------------------
// WorkerFontContext
// ---------------------------------------------------------------------------

/// Per-worker font context accumulated while resolving the families needed by
/// one document.
///
/// Implements [`FontMetrics`] for layout measurement and can produce a `fontdb`
/// database for PNG rasterisation containing only the fonts this worker resolved.
pub(crate) struct WorkerFontContext<'cache> {
    cache: &'cache SharedFontCache,
    /// CSS generic → resolved fontdb family name for this document's fonts.
    generic_assignments: HashMap<String, String>,
    /// Fonts actually resolved for this document (subset of the shared cache).
    resolved_entries: Vec<Arc<FontEntry>>,
}

impl<'cache> WorkerFontContext<'cache> {
    pub(crate) fn new(cache: &'cache SharedFontCache) -> Self {
        Self {
            cache,
            generic_assignments: HashMap::new(),
            resolved_entries: Vec::new(),
        }
    }

    /// Resolve a `@font` CSV value and record its generic assignment (if any)
    /// and its resolved font entry reference.
    pub(crate) fn add_family_csv(&mut self, family_csv: &str) {
        let (generic_assignment, font_entry) = self.cache.resolve_family_csv(family_csv);
        if let Some((generic, family_name)) = generic_assignment {
            self.generic_assignments
                .entry(generic)
                .or_insert(family_name);
        }
        self.resolved_entries.push(font_entry);
    }

    /// Build a `fontdb` database for PNG rasterisation.
    ///
    /// Only the default font and the fonts this worker resolved are registered;
    /// fonts loaded by other workers for other documents are excluded.
    pub(crate) fn build_fontdb(&self) -> fontdb::Database {
        self.cache
            .build_fontdb_for_document(&self.resolved_entries, &self.generic_assignments)
    }

    /// Consume this context and return the CSS generic assignments and resolved
    /// font entries.
    ///
    /// Used when transferring these to a longer-lived owner (such as `Rendered`)
    /// that outlives this context's borrow of the cache.
    pub(crate) fn into_parts(self) -> (HashMap<String, String>, Vec<Arc<FontEntry>>) {
        (self.generic_assignments, self.resolved_entries)
    }
}

impl FontMetrics for WorkerFontContext<'_> {
    fn measure_text_width(&self, text: &str, font: &FontSpec) -> Px {
        self.cache.measure_default(text, font.size().to_f32())
    }
}

// ---------------------------------------------------------------------------
// resolve_font_path
// ---------------------------------------------------------------------------

/// Resolve which font file to use, honouring the documented precedence:
/// `--font` argument, then `TCHART_FONT`, then OS auto-detection.
///
/// Resolution order is defined in `docs/spec/cli.md` §フォント解決の優先順位.
pub(crate) fn resolve_font_path(arg: Option<&Path>) -> Result<PathBuf, CliError> {
    if let Some(path) = arg {
        return ensure_exists(path.to_path_buf());
    }
    if let Some(env_value) = std::env::var("TCHART_FONT")
        .ok()
        .filter(|value| !value.is_empty())
    {
        return ensure_exists(PathBuf::from(env_value));
    }
    find_first_existing_font()
}

// ---------------------------------------------------------------------------
// Font family CSV parsing (used by workers and render_single)
// ---------------------------------------------------------------------------

/// Extract all unique `@font` family CSV values from TCML source text.
///
/// Each worker calls this on its own source text to determine which families
/// it needs to resolve.  The function does **not** aggregate across documents.
pub(crate) fn extract_font_families(tcml_source: &str) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    tcml_source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            let after_keyword = trimmed.strip_prefix("@font")?;
            if !after_keyword.starts_with(char::is_whitespace) {
                return None;
            }
            let value = after_keyword.trim();
            if value.is_empty() {
                return None;
            }
            Some(value.to_owned())
        })
        .filter(|value| seen.insert(value.clone()))
        .collect()
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Strip surrounding double-quotes from a TCML `@font` value.
///
/// TCML allows `@font "Comic Neue"` (quoted) and `@font sans-serif` (unquoted).
/// This function normalises both forms to the bare value without surrounding quotes.
fn strip_font_quotes(value: &str) -> &str {
    let trimmed = value.trim();
    trimmed
        .strip_prefix('"')
        .and_then(|stripped| stripped.strip_suffix('"'))
        .map(str::trim)
        .unwrap_or(trimmed)
}

/// Resolve a family name to a font file path using the OS mechanism.
fn resolve_family_to_path(family: &str) -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        resolve_via_fc_match(family)
    }
    #[cfg(not(target_os = "linux"))]
    {
        resolve_via_directory_scan(family)
    }
}

/// Resolve a family name via `fc-match` (Linux).
#[cfg(target_os = "linux")]
fn resolve_via_fc_match(family: &str) -> Option<PathBuf> {
    // Guard against newline injection.
    if family.contains('\n') || family.contains('\r') {
        return None;
    }
    let output = std::process::Command::new("fc-match")
        .args(["-f", "%{file}\n%{family}", family])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = std::str::from_utf8(&output.stdout).ok()?;
    let mut lines = stdout.lines();
    let path_str = lines.next()?.trim();
    let returned_family = lines.next().unwrap_or("").trim();
    if path_str.is_empty() {
        return None;
    }
    let path = PathBuf::from(path_str);
    if !path.is_file() {
        return None;
    }
    if !CSS_GENERICS.contains(&family) && !returned_family.eq_ignore_ascii_case(family) {
        return None;
    }
    Some(path)
}

/// Resolve a family name by scanning well-known font directories (non-Linux).
#[cfg(not(target_os = "linux"))]
fn resolve_via_directory_scan(family: &str) -> Option<PathBuf> {
    let dirs: &[&str] = &[
        "/System/Library/Fonts",
        "/Library/Fonts",
        "C:\\Windows\\Fonts",
    ];
    let needle = family.replace(' ', "").to_lowercase();
    for directory in dirs {
        let directory_path = std::path::Path::new(directory);
        if !directory_path.is_dir() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(directory_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
                    if stem.replace(' ', "").to_lowercase().contains(&needle) {
                        return Some(path);
                    }
                }
            }
        }
    }
    None
}

/// Extract the first family name from a loaded font via `fontdb`.
fn extract_first_family_name(resolved_font: &ResolvedFont) -> Result<String, CliError> {
    let mut database = fontdb::Database::new();
    database.load_font_data(resolved_font.font_bytes().to_vec());
    database
        .faces()
        .next()
        .and_then(|face| face.families.first())
        .map(|(name, _)| name.clone())
        .ok_or_else(|| FontError::NoFamilyName.into())
}

/// Assign CSS generic family names to `database`.
fn assign_css_generics(
    database: &mut fontdb::Database,
    generic_assignments: &HashMap<String, String>,
    default_family_name: &str,
) {
    let resolved = |key: &str| {
        generic_assignments
            .get(key)
            .cloned()
            .unwrap_or_else(|| default_family_name.to_owned())
    };
    database.set_sans_serif_family(resolved("sans-serif"));
    database.set_serif_family(resolved("serif"));
    database.set_monospace_family(resolved("monospace"));
    database.set_cursive_family(resolved("cursive"));
    database.set_fantasy_family(resolved("fantasy"));
}

fn ensure_exists(path: PathBuf) -> Result<PathBuf, CliError> {
    if path.is_file() {
        Ok(path)
    } else {
        Err(FontError::NotFound(path).into())
    }
}

fn find_first_existing_font() -> Result<PathBuf, CliError> {
    CANDIDATE_FONTS
        .iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
        .ok_or_else(|| FontError::AutodetectFailed.into())
}
