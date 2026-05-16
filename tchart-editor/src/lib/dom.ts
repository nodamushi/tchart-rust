/**
 * Look up an element by id and validate that it is an instance of the
 * expected constructor. Throws on missing id or wrong element type, which
 * is intended as a startup-time crash so the caller never has to silence a
 * cast at the entry point.
 *
 * Use this at app start instead of
 * `document.getElementById(id) as HTMLXxxElement`, which hides both the
 * `null` case and the wrong-type case.
 *
 * @throws Error when no element with the given id is found, or when the
 *   found element is not an instance of `elementConstructor`.
 */
export function requireElement<T extends HTMLElement>(
  id: string,
  elementConstructor: abstract new (...args: unknown[]) => T,
): T {
  const element = document.getElementById(id);
  if (element === null) {
    throw new Error(`requireElement: id=${id} not found`);
  }
  if (!(element instanceof elementConstructor)) {
    // `.name` is used for a development-time crash message. In production
    // builds the class name may be mangled by a minifier; we accept that
    // trade-off rather than inventing a fallback.
    throw new Error(`requireElement: id=${id} is not ${elementConstructor.name}`);
  }
  return element;
}
