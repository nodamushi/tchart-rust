# Rust Coding Rules

Rules AI assistants and reviewers must follow. Each rule reflects a real past violation. Do not invent exceptions; amend this document instead.

---

## 1. Lints

`Cargo.toml` (use `[workspace.lints.clippy]` + `[lints] workspace = true` for workspaces):

```toml
[lints.clippy]
too_many_lines = "deny"
unwrap_used = "deny"
undocumented_unsafe_blocks = "deny"
unused_result_ok = "deny"
module_name_repetitions = "allow"
must_use_candidate = "allow"
exit = "deny"
```

`clippy.toml`:

```toml
too-many-lines-threshold = 30
```

`#[allow(clippy::too_many_lines)]` and `#[allow(dead_code)]` are forbidden (dead_code exceptions: §14).

---

## 2. Naming

- Bool-returning fn/method: must start with `is_` / `has_` / `can_` / `should_` / `check_`. No exceptions.
  - If you want to return success / failed(error), fn shuold return `Result<(), E>` or `Result<(), ()>`.
- **Action methods (those that perform an operation, mutate state, or write to a buffer) MUST start with a verb.** Noun-only names like `attribute`, `write_attr`, `attr` for buffer-writing operations are forbidden — use `write_attribute`, `push_*`, `set_*`, `take_*`, `render_*`, etc.
  - NG (action method as noun): `fn attribute(buf, name, value)` (actually writes), `fn element_width(...) -> Px` (actually computes)
  - OK: `fn write_attribute(...)`, `fn calc_element_width(...) -> Px`
- **Pure getters (read-only accessors that return an owned/borrowed field value)** follow Rust API Guidelines C-GETTER (<https://rust-lang.github.io/api-guidelines/naming.html>): noun-named, no `get_` prefix. `fn position(&self) -> Point`, `fn content(&self) -> &Content`, `fn bounding_box(&self) -> Rect` are all OK. The `get_` prefix is forbidden.
- **External-trait required methods are exempt** when the signature is fixed by an external crate (e.g. `std::fmt::Display::fmt`, `std::fmt::Debug::fmt`, `std::hash::Hash::hash`, `std::cmp::PartialEq::eq`, `serde::Serialize::serialize`). Required signatures inside `impl Trait for Type` blocks are not graded against §2.
- No abbreviations in any identifier. NG `doc`, `hw`, `gp`, `attr`, `ch`, `lit`, `px` (when used as identifier name, not type), `bbox`, `geom`, `idx`, `iter` (as variable name), `cnt`, `pos`, `coord`, `len` (as variable name — the method `len()` is exempt) / OK `document`, `hello_world`, `global_params`, `attribute`, `character`, `literal`, `bounding_box`, `geometry`, `index`, `iterator`, `count`, `position`, `coordinate`, `length`. Exception: widely accepted (`www`, `WDT`), and project-established type names (`Px` as the pixel newtype is grandfathered). **Domain shorthand from external sources ("everyone in graphics calls it bbox") is NOT a valid reason to keep an abbreviation — write the full word.**
- No single-letter identifiers. NG `w`, `h`, `e`, `s` / OK `width`, `height`. Exception: coordinates `x`, `y`, `z` in geometric contexts.

---

## 3. Constructors and Type Design

### 3.1 Constructors are associated functions or `From`

A function returning `T` must live in `impl T` (or `impl From<...> for T`), never as a free function.

- NG: `fn build_foo(...) -> Foo`
- OK: `impl Foo { fn new(...) -> Self }`, `impl From<&Bar> for Foo`

### 3.2 Construction priority

1. `Default` if possible.
2. Single `fn new(...)` that validates inputs.
3. Builder pattern for many arguments.

Public fields to allow post-construction mutation are forbidden.

### 3.3 Constants over functions

Do not write a function when a constant suffices.

- NG: `fn make_foo_default() -> Foo { Foo { ... } }`
- OK: `impl Foo { const DEFAULT: Self = Self { ... }; }`

"I do not want to edit another file" is not a valid reason.

### 3.4 NewType

- Use `derive_more` for boilerplate.
- Implement your own validation method.
- When deserializing with serde, **call the validation method immediately after deserialization**. Skipping defeats the purpose.

### 3.5 Inherent impl lives with the type

`impl Foo { ... }` (non-trait `impl`) belongs in the same module — and normally the same file — as the definition of `Foo`. Do not scatter inherent `impl` blocks for the same type across multiple modules to avoid editing the file where `Foo` is defined.

- NG: needing a new method on `Foo` while working in mod bar, and writing `impl Foo { fn new_method(...) { ... } }` inside `mod bar` because opening `foo.rs` feels like extra work.
- OK: open `foo.rs` and add the method to the existing `impl Foo` block.

Legitimate reasons to split inherent `impl` across files (closed list):

- `#[cfg(feature = "...")]` or `#[cfg(test)]` gating where the gated methods are cleanly separable.
- The type is genuinely large and multi-faceted (e.g. an AST node, a world-state struct), and splitting `impl` blocks by responsibility aids navigation rather than hindering it.

"I am editing a different file right now"/ "I want to use `pub(super)`" are not on this list. If none of the legitimate reasons apply, edit the file where the type is defined.

---

## 4. Visibility

### 4.1 Smallest visibility, widened on demand

```
private → pub(super) → pub(in path) → pub(crate) → pub
```

Default to private. Widen one step at a time, only when an actual caller requires it. Never start `pub` and narrow.

### 4.2 Invalid reasons to widen

- "I might need it later."
- "Another module might want it."
- "For symmetry with other functions in this file."
- "The test will need it." Tests live in a sibling `tests.rs` (§8) and already see private items.

### 4.3 Cluster audit

When you find one over-exposed item, audit the entire file and module. Over-exposure clusters.

### 4.4 Scope

Applies to `fn`, `struct`, `enum`, `trait`, `mod`, `const`, `type`. Field rules (§5) are stricter.

---

## 5. Struct Fields

### 5.1 Fields are private

`pub`, `pub(crate)`, `pub(super)`, `pub(in ...)` on fields are all forbidden.

### 5.2 No mechanical getters

Writing `pub(crate) fn field(&self) -> &T` for every field is the Java POJO anti-pattern and counts as a violation in spirit. **Tell, don't ask** — callers ask the struct to *do* something, not to extract internals.

A getter is justified only when (a) an external caller has a concrete, named use for one specific field, and (b) the operation cannot be moved onto the struct itself. Each one is justified individually. "It's private, so I added a getter" is not a justification.

### 5.3 Exceptions (closed list)

*Do not look for loopholes just to make things easier.*

Public fields are allowed only for:

1. There must be no methods that take `&mut self`.
  - If a method that takes `&mut self` becomes necessary later, define accessors for all fields and make all fields private.
  - No exceptions are allowed.
  - Humans tend to try to restrict the scope of `mut`, but AI tends to expand it just to make things easier. Because of this laziness, this project generally requires the definition of accessors.
2. There must be no methods of the Builder pattern (`fn set_x(mut self, x) -> Self`).
  - No Builder pattern-concealing methods that return a separate object created solely to circumvent the function signature are permitted.
3. No matter what changes occur externally, there must be no inconsistencies in the internal state. All fields are independent and semantically unrelated to one another.
  - `Point(x,y)` is permitted because x and y are independent in all cases.
  - `Buf(len, Box)` must not grant visibility under any circumstances, as they are mutually dependent.
  - If dependencies arise between fields, define accessors for all fields and make all fields private. No exceptions are allowed.
4. **Geometric/mathematical value types** with no inter-field invariants and universally understood field names (e.g. 2D point `x`/`y`).
5. **Parameter object** for a single function call. No methods (except `Default`/`new`), no invariants, no second use site. Name ends in `Args` / `Params` / `Options`.
6. **CLI parse result** from a derive macro (e.g. `#[derive(Parser)]`).
7. **serde DTO** for direct deserialization. Must be converted into a domain type with validation immediately after; never passed around.
8. **One-shot result struct**: a struct that exists solely as the return value of one specific computation function, is never stored in a field, never aliased, and is consumed (destructured by the caller) within the same module. Public fields on such a struct beat a fistful of mechanical getters. (Example: `StackingResult` returned by `stack_lines()` and immediately destructured by `compute_chart_dimensions`.)
9. **Tuple newtype with banned `.0` access**: `struct X(pub Y)` forms are allowed *only when* §9.5 is enforced — i.e. `.0` access outside `impl X`/`match` is prohibited project-wide. The `pub` on `.0` exists to permit `X(value)` constructor literals and `match X(value)` destructuring; it is not an invitation to read `value.0` from caller code (use `to_*` / `as_*` / arithmetic / `From` instead). (Example: `Px(pub f32)`.)

### 5.4 Accessors return references

When you do justify an accessor under §5.2, return `&T`, not `T`.

- OK: `fn name(&self) -> &str`
- OK: `fn position(&self) -> &Position`
- OK for `Copy` types: `fn count(&self) -> usize` (returning by value is fine for `Copy`)
- NG without justification: `fn name(&self) -> String` (clones on every call)

Returning a reference forces the caller to confront borrow conflicts at the call site rather than hiding them. This is intentional.

### 5.5 Do not paper over borrow conflicts with `clone()`

Using `clone()` solely to dodge a borrow-checker error is forbidden. The borrow conflict is signalling that the caller is mixing immutable and mutable access in a way that needs to be redesigned, not silenced.

- NG (cloning to dodge the conflict):
```rust
  let x = mutdata.position().clone();  // only reason for clone: avoid the borrow
  mutdata.set_y(x.y + 1);
```
- OK (move the operation onto the type — Tell, don't ask):
```rust
  mutdata.shift_y(1);
```

`clone()` is permitted when you actually need an independent owned copy for downstream use, - not when its only role is to make the borrow checker quiet.

The combined intent of §5.4 and §5.5 is to keep §10 (`&mut` discipline) honest. Accessors that return owned values, or callers that clone away conflicts, both let `&mut` spread silently. This project closes both escape hatches.



### 5.6 Methods, not free functions with `&mut Arg`

- NG (free function in some other module):
```rust
fn update(arg: &mut Arg, source: &Foo) { ... }
```
- NG:
```rust
impl Foo {
    fn update(&self, arg: &mut Arg) { ... }
}
```
- OK (method on `Arg`, in the file where `Arg` is defined):
```rust
impl Arg {
    fn update(&mut self, source: &Foo) { ... }
}
```

Don't scatter `mut` all over the place just to make things easier.

If there are two or more `&mut` arguments, ask yourself whether you really need `mut` in the first place.

**Exception**: `fn op(&self, out: &mut Arg)` is permitted when the operation is naturally defined per `&self` type — i.e. it is `Self`'s knowledge of how to act on `Arg`, not `Arg`'s knowledge of how to receive a `Self`. The operation must go through a trait so that `Arg`'s public API remains a single `&mut self` method.

```rust
trait DrawOn {
    fn draw_on(&self, target: &mut Arg);
}

impl Arg {
    fn draw<T: DrawOn>(&mut self, source: &T) {
        source.draw_on(self);
    }
}
```

The test for this exception: would defining the operation as `&mut self` on `Arg` force `Arg` to know about every `Self` type that might call it? If yes, the operation belongs on `&self` and this exception applies. If no, write it as `&mut self` on `Arg` per the main rule.

### 5.7 Project-specific exceptions

See `docs/coding/rust-project.md`. **Do not invent new exception categories.**

---

## 6. Errors / CLI / Crates

- Errors: count **variants** that need `Display` formatting (across all error enums in the file/crate, including nested ones reached via inner enums). If the total exceeds 10, use `thiserror`. Otherwise hand-rolled `impl Display` / `impl Error` is fine.
  - Do **not** count enum *types*. "We only have 3 enums so thiserror is unnecessary" is the wrong reading. One 30-variant `CliError` alone justifies `thiserror`; three small enums of 4 variants each totalling 12 variants also justify it.
  - When in doubt, count: open `error.rs`, count match arms in every `impl Display`, sum them up.
- Complex CLI parsing: use `clap`. Fixed positional args only: parse yourself.
- NewType boilerplate: use `derive_more`.

---

## 7. Input Safety

### 7.1 Never trust external input

Range-check all numeric input. Do not feed input directly into calculations; assume it can panic.

### 7.2 Containment via NewType

Receive input as an unsafe NewType and resolve via methods. Receiving as `String` and "resolving" by passing through a free function is forbidden.

- Violation: `fn handle(input: String) { let escaped = escape(input); }`
- OK: `fn handle(input: UnsafeInput) { let escaped = input.sanitize(); }`

### 7.3 No raw strings to dangerous APIs

Never pass unvalidated strings to `Command` etc. Check for unescaped newlines and similar.

### 7.4 Public library APIs

Accept via `AsRef` / `Into`: `pub fn foo(password: impl Into<Password>)`.

---

## 8. Tests

- Unit tests in a sibling `tests.rs` module. Integration tests in `tests/`.
- Inline `#[cfg(test)] mod tests { ... }` is forbidden. Child modules see parent's private items, so inline modules are never necessary.

### 8.1 Test code is graded leniently

Test code is judged on whether the scenario is clear, not on style purity. Cosmetic findings (length, local-variable naming, minor style inconsistency, light boilerplate) should not block a test. Production-side rules around safety, correctness, and external visibility (no `unwrap`, no local-ID leakage in test names, no Japanese in source, no `#[allow]` shortcuts, no `pub` widening) still apply.

---

## 9. Code Structure

### 9.1 `pub` first

In each file, write `pub` items before private ones.

### 9.2 No local-variable bloat

Do not assemble logic by lining up many local variables. Define a struct and operate on it via methods.

### 9.3 No deep field chains

Avoid `a.b.c.d.e`. Bind intermediates to named variables or add helper methods.

### 9.4 `if let` + `&&`, not nested `if`

- NG: `if let Foo::Bar(x) = v { if cond(x) { ... } }`
- OK: `if let Foo::Bar(x) = v && cond(x) { ... }`

### 9.5 No tuple-field access outside the type's `impl`

`.0`, `.1` are permitted **only** inside `impl MyType` or `match` / `let` destructuring. **No exceptions.** Outside those contexts, every `.0` / `.1` is a violation regardless of how short or "obvious" the access looks. Implement an operator (`Add`, `Sub`, etc.), a `to_*` / `as_*` conversion, or a verb method on the type.

- NG: `expected.0`, `value.0 + other.0`, `format!("{}", point.0)`
- OK inside `impl Px`: `self.0 * 2.0`
- OK in pattern: `let Px(value) = px;`, `match px { Px(x) if x > 0.0 => ... }`

This rule is the precondition for §5.3(6): tuple newtypes get to keep `pub` on `.0` only because every external read site is required to go through a method.

### 9.6 No slice index access where avoidable

Use slice operations, iterators, pattern matching, or slicing.

### 9.7 `matches!` is not for comparison

Derive `PartialEq` and use `==` / `!=`.

- NG: `!matches!(v, MyEnum::Foo)`
- OK: `v != MyEnum::Foo`

`matches!` is allowed only when destructuring is needed: `matches!(x, Foo::Bar(_))`.

---

## 10. `&mut`

- Minimize. Do not pass a `&mut` argument down into other functions.
- At most `&mut self` plus one other `&mut` argument. Three or more is a design failure.
- No output parameters: NG `fn emit(out: &mut Vec<T>)` / OK return `Vec<T>` or `impl Iterator<Item = T>`.
- Threading `&mut` through multiple functions to reduce line count is forbidden — it obscures who owns the mutation.

---

## 11. No C-style Rust

Forbidden:

- Output parameters.
- Free functions that belong on a type.
- Manual struct decompose-and-recompose (implement `Add`, `Sub`, `From`).
- Repeated deep field chains (helper methods or named bindings).
- `matches!` as comparison (derive `PartialEq`).

**Test**: if your Rust transliterates to C with minimal effort, rewrite it.

---

## 12. Iterators

### 12.1 Return iterators, not output buffers

Prefer `impl Iterator<Item = T>` over `&mut Vec<T>`. Callers use `collect()` or `extend(...)`.

### 12.2 Never rebuild a `Vec` with `for` + `push`

- NG:
  ```rust
  let mut v = Vec::new();
  for d in data { v.push(foo(d)); }
  ```
- OK: `let v: Vec<_> = data.iter().map(foo).collect();`

For `Result`-yielding iterators with short-circuiting: `iter.collect::<Result<Vec<_>, _>>()?`.

### 12.3 Use adaptors, not manual loops

Replace empty-Vec + loop + conditional push with `map` / `filter` / `filter_map` / `flat_map` / `fold` / `try_fold` etc.

- NG:
  ```rust
  let mut output = Vec::new();
  for item in items { if item.is_active() { output.push(item.value() * 2); } }
  ```
- OK: `items.iter().filter(|i| i.is_active()).map(|i| i.value() * 2).collect()`

---

## 13. Line Count and Macros

`too-many-lines-threshold = 30`'s purpose is **readability**, not mechanical splitting.

- Long or repetitive code: define readable macros.
- Splitting in half without thought does not help readability.
- Splitting that scatters `&mut` (§10) is forbidden.

---

## 14. Dead Code

`#[allow(dead_code)]` is forbidden. Remove unused functions.

Exceptions:

1. Code unused under some feature configurations.
2. Functions absolutely necessary for testing.

Even then, prefer `#[cfg]` to avoid producing dead code in the first place.

---

## 15. `std` Prefix

Forbidden: importing a top-level `std` module just to strip the prefix.

- NG:
  ```rust
  use std::io;
  use std::fs;
  fn read() -> io::Result<fs::File> { ... }
  ```

OK forms:

- Import the leaf type: `use std::fs::File; use std::io::Result;`
- Local `use` inside the function: `fn read() -> std::io::Result<()> { use std::io::{Error, ErrorKind}; ... }`
- Inline full path: `fn read() -> std::io::Result<()>`

This rule does **not** require fully-qualified paths everywhere. `Option`, `Result`, `Vec`, `Box`, `String` etc. are in the prelude — writing `std::option::Option` is wrong, not "compliant." The ban is specifically on stripping to one-segment names like `io::` / `fs::` / `sync::` / `mem::` / `cmp::`.

---

## 16. Documentation

- All public items (`pub fn`, `pub struct`, `pub enum`, `pub trait`, ...) require doc comments.
- Internal items: comment when intent is not obvious from the name.
- `unsafe` blocks require a `// SAFETY:` comment (`undocumented_unsafe_blocks = "deny"`).

### 16.1 Do not embed mutable cross-references in source comments

Comments that point at unstable artifacts rot the moment those artifacts move. Forbidden in any source comment (`//`, `///`, `//!`):

- **References to `docs/coding/rust.md` itself** — section numbers (`§5.6`, `§3.1(7)`, `Per docs/coding/rust.md §X`) change every time this document is reorganised. Explain the *reason* in plain language; do not cite the rule number.
- **Local identifiers** — see AGENTS.md `# No Local IDs`.
- **"Replaces the prior X" / "previously this was Y"** rationale that only makes sense relative to a deleted version. Document what the code *does* now, not what it used to be.

Stable identifiers are fine:

- Spec section names from `docs/spec/*.md` (`docs/spec/types.md` §2.4, `docs/spec/tcml-format.md` §「@clock」). These are the *spec*, not implementation rules, and are referenced by their named heading rather than a serial number.

If the rationale truly needs the rule number, that means the code is fragile to the rule changing — which itself is a smell. Restructure so the constraint is self-evident from the code shape.

---

## 17. Do Not Create Pointless Functions

A function costs a name, a signature, indirection, and a jump for the reader. It must earn the cost.

**The axis is whether the name carries information the call site would lose.** Line count is not the test.

A function is justified by at least one of:

1. **Reuse** — used at two or more sites (or about to be).
2. **Naming** — the name carries meaning the inlined expression would not. Valid even for one-line bodies. OK: `fn is_weekend(d: Day) -> bool { d == Day::Sat || d == Day::Sun }`. NG: `fn add_one(n: i32) -> i32 { n + 1 }`.
3. **Encapsulation** — enforces an invariant or hides a representation.
4. **Trait/API surface** — required by a trait, public API, or callback signature.

If none apply, inline it.

Typical violation — single call site, single expression, name says nothing the inlined form does not:

```rust
fn parse_attr_list(value: &str) -> Result<SvgAttrList, ParseError> {
  Ok(tokenize_attrs(value)?.collect())
}
// caller: let list = parse_attr_list(value)?;
```

Inline:

```rust
let list: SvgAttrList = tokenize_attrs(value)?.collect();
```

---

## 18. Review Checklist

**Design**

- Duplication: does the logic already exist? Search before adding.
- Lazy `pub`: did you widen visibility instead of restructuring?
- Type not defined: are you manipulating raw strings/numbers where an intermediate `struct`/`enum` belongs? (e.g. HTML built by `String` concat → DOM-like type; `&str` password → `&Password`.)
- Correctness: does it match the spec?

**Safety**

- Panic risk in numeric calculations on untrusted input?
- Untrusted strings reaching `Command` etc.?

**Visibility**

- Any `pub` without a real call site?
- Found one over-exposure → did you audit the file and module?

**Implementation**

- C-style Rust (§11)?
- `&mut` scattered?
- Local-variable bloat where a struct fits?
- `Vec` built by `for`+`push` instead of `collect()` / adaptors (§12)?
- Slice index access avoidable?
- `.0` / `.1` outside the type's own `impl`?
- Pointless wrapper function (§17)?

**Quality**

- Public items documented?
- Tests exist and live in separate files?
- Comments where intent is non-obvious?
