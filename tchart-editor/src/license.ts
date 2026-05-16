import { LICENSE_GROUPS, type LicenseGroup, type LicenseLibrary } from "./generated/licenses";
import { closeHelpModal } from "./help";

const MODAL_ID = "license-modal";
const BODY_OVERFLOW_BACKUP = "data-tcml-prev-overflow";

function renderLibraryRow(library: LicenseLibrary): HTMLElement {
  const row = document.createElement("li");
  row.className = "license-library";

  const name = document.createElement("span");
  name.className = "license-name";
  name.textContent = library.name;

  const version = document.createElement("span");
  version.className = "license-version";
  version.textContent = library.version;

  row.append(name, version);

  // why: MIT / BSD compliance requires that each library's individual
  // copyright notice ride along with the license body. The shared body is
  // deduplicated at the group level, so the per-library copyright lives
  // here on the library row.
  if (library.copyright !== "") {
    const copyright = document.createElement("span");
    copyright.className = "license-copyright";
    copyright.textContent = library.copyright;
    row.append(copyright);
  }
  return row;
}

/**
 * Render one license-body group: SPDX label, license body shown once, and
 * the list of libraries distributed under that body. DOM APIs (not
 * innerHTML) are used for the license text so the raw body cannot be
 * interpreted as HTML.
 */
function renderGroup(group: LicenseGroup): HTMLElement {
  const wrapper = document.createElement("section");
  wrapper.className = "license-group";

  const header = document.createElement("div");
  header.className = "license-group-header";

  const spdx = document.createElement("span");
  spdx.className = "license-spdx";
  spdx.textContent = group.spdx;
  header.append(spdx);

  const body = document.createElement("pre");
  body.className = "license-text";
  body.textContent = group.body;

  const list = document.createElement("ul");
  list.className = "license-library-list";
  for (const library of group.libraries) {
    list.append(renderLibraryRow(library));
  }

  wrapper.append(header, body, list);
  return wrapper;
}

/**
 * Open the license modal. No-op if it is already open. The modal can be
 * closed via the Escape key or the close button. While open, the body
 * `overflow` is forced to `hidden` so background content (toolbar / editor /
 * preview) cannot scroll; the previous value is restored on close.
 *
 * @remarks
 * If the Help modal is currently open, it is closed first. The spec forbids
 * stacking both dialogs at once, so opening the License modal implicitly
 * closes the Help modal as a side-effect.
 */
export function openLicenseModal(): void {
  if (document.getElementById(MODAL_ID) !== null) return;

  // why: Help and License are sibling dialogs. The spec forbids stacking
  // both visible at once; closing Help here is the simplest reconciliation
  // and keeps the rule enforced regardless of who calls openLicenseModal.
  closeHelpModal();

  const overlay = document.createElement("div");
  overlay.id = MODAL_ID;
  overlay.className = "license-overlay";

  const dialog = document.createElement("div");
  dialog.className = "license-dialog";
  dialog.setAttribute("role", "dialog");
  dialog.setAttribute("aria-modal", "true");
  dialog.setAttribute("aria-labelledby", "license-modal-title");

  const header = document.createElement("div");
  header.className = "license-header";

  const title = document.createElement("span");
  title.id = "license-modal-title";
  title.className = "license-title";
  title.textContent = "Third-party licenses";

  const closeButton = document.createElement("button");
  closeButton.type = "button";
  closeButton.className = "license-close";
  closeButton.setAttribute("aria-label", "Close licenses");
  closeButton.textContent = "×";

  header.append(title, closeButton);

  const body = document.createElement("div");
  body.className = "license-body";
  for (const group of LICENSE_GROUPS) {
    body.append(renderGroup(group));
  }

  dialog.append(header, body);
  overlay.append(dialog);
  document.body.append(overlay);

  // why: backing up the previous value avoids stomping on any future
  // body-overflow override applied elsewhere; close paths read this back.
  document.body.setAttribute(BODY_OVERFLOW_BACKUP, document.body.style.overflow);
  document.body.style.overflow = "hidden";

  // why: the Escape handler is registered on `document`, so the close button
  // and backdrop-click paths must also remove it; otherwise listeners pile
  // up each time the modal is reopened.
  const closeAndCleanup = () => {
    document.removeEventListener("keydown", onKeyDown);
    closeLicenseModal();
  };
  const onKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      closeAndCleanup();
    }
  };

  closeButton.addEventListener("click", closeAndCleanup);
  overlay.addEventListener("click", (event) => {
    if (event.target === overlay) closeAndCleanup();
  });
  document.addEventListener("keydown", onKeyDown);
}

/**
 * Remove the license modal from the DOM if it is open and restore the
 * previous body overflow. Listener cleanup is handled inside the closure
 * set up by `openLicenseModal`.
 */
export function closeLicenseModal(): void {
  const overlay = document.getElementById(MODAL_ID);
  if (overlay === null) return;
  overlay.remove();
  const previous = document.body.getAttribute(BODY_OVERFLOW_BACKUP);
  document.body.style.overflow = previous ?? "";
  document.body.removeAttribute(BODY_OVERFLOW_BACKUP);
}

/**
 * Whether the license modal is currently mounted in the DOM.
 */
export function isLicenseModalOpen(): boolean {
  return document.getElementById(MODAL_ID) !== null;
}
