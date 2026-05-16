# TypeScript Coding Rules

Each rule closes a path AI has actually taken to dodge work. Do not invent exceptions; amend this document instead. TS has no borrow checker, so the discipline equivalent to Rust's is distributed across four layers: (a) `tsconfig` strictness, (b) Branded Types for nominal typing, (c) `readonly` by default, (d) minimized `export`.

---

## 1. The Type System as a Discipline Device

### 1.1 No `any`

`any` disables the type system entirely. For external inputs of unknown type, take them as `unknown` and validate before use.

- NG: `function foo(input: any) { ... }`
- NG: `const data = JSON.parse(text); data.x.y`
- OK: take as `unknown`, validate via type predicate or Zod schema

### 1.2 Do Not Silence Casts

Permitted casts are limited to:

1. `as const` — literal narrowing
2. `as unknown` — entry point before validation
3. `satisfies` — confirms type conformance while preserving inferred type. Prefer over `as`
4. Casts inside a validate function to attach a Branded Type (§2.1)

All other `as X` is forbidden.

- NG: `const user = data as User;`
- NG: `(value as any).foo`
- OK: `const config = { ... } satisfies Config;`
- OK: `if (isUser(data)) { /* data: User */ }`

### 1.3 No Non-Null Assertion `!`

`!` is the one-liner version of "trust me, this isn't undefined." Use type guards, `??`, or early return.

### 1.4 No Bare `@ts-ignore` / `@ts-expect-error`

Without a **concrete external cause + trackable link** on the same line, suppression is forbidden.

- NG: `// @ts-ignore`
- NG: `// @ts-ignore — TS is wrong here`
- OK: `// @ts-expect-error — type bug in upstream lib XXX@1.2, issue#NNN, no workaround`

### 1.5 Always Validate `unknown` at the Entry Point

External inputs (`fetch`, `JSON.parse`, IPC, user forms, `localStorage`) are taken as `unknown` and immediately validated into a branded type.

```ts
const raw: unknown = JSON.parse(text);
const user = userSchema.parse(raw);
```

Define validation as a single Zod schema; do not duplicate validation logic. Whether to use Zod or not must be uniform across the project. No mixing.

### 1.6 tsconfig

The following are required. Relaxing any of them requires user approval:

```json
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true,
    "noImplicitOverride": true,
    "noFallthroughCasesInSwitch": true,
    "noImplicitReturns": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true
  }
}
```

---

## 2. Branded Types and Discriminated Unions

### 2.1 Brand at the Entry Point, Pass Branded Types Between Functions

When using Zod, use `.brand()`. This is the standard pattern:

```ts
const EmailSchema = z.string().email().brand<"Email">();
type Email = z.infer<typeof EmailSchema>;

function sendMail(to: Email, ...) { ... }

const email = EmailSchema.parse(userInput);
sendMail(email);          // OK
sendMail(userInput);      // type error
sendMail("foo" as Email); // §1.2 violation, reject in review
```

For internal types that do not go through Zod (`RowVersion`, `Etag`, `Hash`, etc.), the hand-rolled pattern is permitted:

```ts
type RowVersion = string & { readonly __brand: "RowVersion" };

function parseRowVersion(input: string): RowVersion | ParseError {
  if (!isRowVersionShape(input)) return new ParseError("invalid row version");
  return input as RowVersion; // §1.2 permitted cast 4
}
```

Makes "pass raw string straight to the function" impossible at the type level.

### 2.2 Represent State as a Discriminated Union

```ts
type Order =
  | { kind: "draft"; ... }
  | { kind: "submitted"; submittedAt: Date; ... }
  | { kind: "shipped"; trackingId: TrackingId; ... };

function describe(order: Order): string {
  switch (order.kind) {
    case "draft": return "...";
    case "submitted": return "...";
    case "shipped": return "...";
    default: { const _exhaustive: never = order; return _exhaustive; }
  }
}
```

Long chains of `if (order.kind === "submitted") { ... }` are forbidden. Force `switch` and make it exhaustive.

### 2.3 Pick One Error Strategy Project-Wide

Choose either `throw` or Result types, and commit to it. No mixing.

If you choose Result types, you must enable `eslint-plugin-neverthrow`'s `must-use-result`. TS has no equivalent of Rust's `#[must_use]`; without the lint, discarded Result return values cannot be caught, making Result types weaker protection than exceptions. If you cannot fail CI on this lint, use exceptions.

When using neverthrow, do not define your own `type Result<T, E> = ...`. The lint is bound to the library's type and will not fire on your own type.

Converting one to the other at boundaries (HTTP handlers, message queue consumers) is permitted.

---

## 3. `readonly` by Default

### 3.1 Fields Are `readonly` by Default

```ts
class Order {
  readonly id: OrderId;
  readonly items: ReadonlyArray<Item>;
}
```

- Violation: `class Order { items: Item[]; }`
- Violation: `function process(order: Order) { order.items.sort(...); }`
- OK: take as `ReadonlyArray<Item>`; if modification is needed, `[...items].sort(...)` is a deliberate copy

### 3.2 Do Not Mutate Arguments

Mutating function arguments is forbidden. Return a new value.

- NG: `function update(state: State) { state.x = 1; }`
- OK: `function withUpdatedX(state: State): State { return { ...state, x: 1 }; }`

### 3.3 Mutation Only Through Explicit Methods on the Class

Spreading into a copy and modifying it outside the class (`{ ...obj, x: ... }`) is forbidden. If a copy is needed, an explicit method on the class returns a new instance.

- Violation (raw spread mutation outside the class):
```ts
const order2 = { ...order, status: "shipped" };
```

- OK (through a class method):
```ts
class Order {
  ship(trackingId: TrackingId): Order {
    return new Order({ ...this, kind: "shipped", trackingId });
  }
}
const shipped = order.ship(tid);
```

Same intent as Rust's rule against silencing borrow conflicts with `clone()`. If you copy, the method name on the type must express *why* the copy is needed.

### 3.4 Prefer `const` Over `let`

`let` is for cases where reassignment is genuinely required. Accumulator-style `let result = []; for (...) result.push(...)` is forbidden by §9.

---

## 4. Naming

### 4.1 Booleans Start With `is` / `has` / `can` / `should` / `check`

- NG: `function valid(x): boolean`
- OK: `function isValid(x): boolean`

If you want to express success/failure, return a Result type, not a bool.

### 4.2 Methods and Functions Start With a Verb

Including pure accessors, method names start with a verb. Choose names consistent with TS/JS standard APIs (`Map.get`, `URLSearchParams.get`, `document.getElementById`, `element.getAttribute`).

- NG: `function attribute(buf, name, value)` (actually writes)
- NG: `function width(...)` (actually computes)
- NG: `function name(): string` (an accessor, but not a verb)
- OK: `function writeAttribute(buf, name, value)`
- OK: `function calcElementWidth(...): Px`
- OK: `function getName(): string`
- OK: `function fetchUser(id): Promise<User>`

Rust's rule banning the `get_` prefix does not apply to TS. TS standard APIs use `get` pervasively; mechanically importing the Rust rule breaks alignment with the standard library.

TS getter syntax (`get name() { ... }`) is a separate matter. Getter syntax forces the caller to write `obj.name`, which hides whether side effects occur. **Limit getter syntax to pure accessors with no side effects.** For anything involving computation or I/O, use a regular method (`getName()`).

### 4.3 No Abbreviations or Single Letters

Abbreviations and single-letter identifiers are forbidden. As a general principle, the only exceptions are (a) shorthands established in the target ecosystem (`id`, `url`, `xml`, `api`, `jwt`, `aws`, `ui`, `cli`, `tcp`, `tls`, `dom`, etc.), (b) coordinates `x`, `y`, `z`, and (c) loop variable `i` in short blocks.

- NG: `doc`, `attr`, `ch`, `idx`, `iter`, `cnt`, `pos`, `coord`, `len`, `bbox`, `geom`, `w`, `h`, `e`, `s`, `tmp`
- OK: `document`, `attribute`, `character`, `index`, `iterator`, `count`, `position`, `coordinate`, `length`, `boundingBox`, `geometry`, `width`, `height`, `event`, `state`, `temporary`

"The field calls it `bbox`" and "shorter is more readable" are rejected. An abbreviation is acceptable only if ECMAScript / Web Platform / Node.js standard APIs adopt it. "Industry convention" is not the standard library.

---

## 5. Minimize `export`

### 5.1 Default to No Export

Add `export` only when a call site actually exists, at the smallest unit. Exporting first and narrowing later is forbidden.

### 5.2 Invalid Reasons to Widen

- "I might need it later"
- "Another module might want it"
- "For symmetry"
- "The test needs it" — tests can import existing exports from a sibling `*.test.ts`. If you need to see internals, suspect that the production-side responsibility split is wrong

### 5.3 Over-Exposure Clusters

If you find one, audit every `export` in the same file/module.

---

## 6. Class Fields and Object Types

### 6.1 No Mutable Public Fields

```ts
class Foo {
  count: number;             // NG: public mutable
  public name: string;       // NG
  readonly position: Point;  // OK (readonly public)
  private value: number;     // OK (compile-time private)
  #internal: string;         // OK (runtime private)
}
```

For things that genuinely must be hidden, use `#field` (runtime private). `private` is compile-time only and can be defeated with `as any`.

### 6.2 No Mechanical Getters (Tell, Don't Ask)

Spamming `getFoo() { return this.#foo; }` for every field is the Java POJO anti-pattern. Each getter must be individually justified by a named external use case.

If you cannot justify it, put a verb on the class (`order.ship(...)`, `cart.applyDiscount(...)`) rather than extracting state and reassembling it outside.

### 6.3 Closed List of Permitted Public Fields

Public fields are allowed only in:

1. **Fully `readonly` value types** — methods return new instances; no state change
2. **Geometric / mathematical value types** — `Point { x, y }`, no inter-field invariants
3. **Options / Args / Params objects** — argument aggregation for a single function; no methods
4. **Discriminated union variants** — the union itself is the public API
5. **JSON DTOs** — the intermediate form immediately after deserialization; **convert to a domain type at once** and do not pass around
6. **One-shot return values** — destructured by the caller within the same module

### 6.4 No External Access to Tuple Newtype Internals

```ts
class Px {
  constructor(public readonly value: number) {}
  add(other: Px): Px { return new Px(this.value + other.value); }
}
```

The `public readonly value` exists **to permit constructor literals (`new Px(3)`) and destructuring**, not to be read from outside. If reading is needed, add `toNumber()` / `as*()` / `From`-style methods / operator methods (`add`, `sub` in place of `Add`, `Sub`).

- NG: `expected.value`, `a.value + b.value`, `` `${point.value}` ``
- OK (inside the class): `this.value * 2`
- OK: `expected.toNumber()`, `a.add(b)`

### 6.5 Methods Live With the Type

Defining methods of a class in a separate file (extension-style) is forbidden. When you edit the class, open the file the class lives in.

---

## 7. Mutation Discipline

### 7.1 No Argument Mutation

Return a new value.

- NG: `function emit(out: T[]): void { out.push(...); }`
- OK: `function emit(): T[] { return [...]; }`
- OK: `function* emit(): IterableIterator<T> { ... }`

### 7.2 At Most One Mutable Accumulator Per Function

If you have more than one, the design is wrong. Extract to a class.

### 7.3 Do Not Thread Mutation Across Functions

Passing `let counter` through a closure and `++`-ing it from multiple sites is forbidden. State stays inside a class; operations are methods.

---

## 8. Comparison and Equality

### 8.1 Only `===` and `!==`

`==` / `!=` produce bugs through type coercion. Handle null and undefined as follows:

- Just want to test: `x === null || x === undefined`
- Provide a default if nullish: `x ?? defaultValue` (replacement, not a test)
- Short-circuit if nullish: `x?.foo` (optional chaining)

Do not use `x == null`. It is behaviorally equivalent to `x === null || x === undefined`, but permitting `==` anywhere defeats the goal of catching other `==` bugs (`0 == ""`, `null == 0`, etc.).

### 8.2 Exhaustive Switch

See §2.2. In switches over a discriminated union, the `default` performs a `never` check. Long `if-else` chains are forbidden.

### 8.3 Do Not Repurpose Regex or `includes` for Value Comparison

Writing `status.includes("ship")` to detect shipped status is forbidden. Use the discriminated union (`kind === "shipped"`).

---

## 9. Arrays and Iteration

### 9.1 No `for` + `push`

- NG: `const result = []; for (const x of data) result.push(f(x));`
- OK: `const result = data.map(f);`

### 9.2 Use Array Methods

Use `map` / `filter` / `flatMap` / `reduce`. Filter conditionally before mapping:

- OK: `items.filter(i => i.isActive).map(i => i.value * 2)`

For short-circuiting over Result-yielding operations, use `reduce` or an explicit loop with early return.

### 9.3 Avoid Index Access (`arr[i]`)

Use `for...of` / `find` / destructuring. `arr[i]` is acceptable **only when the index itself carries meaning** (zip, matrix ops). Under `noUncheckedIndexedAccess`, results are `T | undefined`, so an undefined check is mandatory when you do use it.

Use `arr.at(i)` only when negative indices are required. If they are not, `arr[i]` is sufficient (both yield `| undefined`).

### 9.4 Return Arrays or Iterators; No Output Parameters

Same as §7.1.

---

## 10. Functions, Length, Arguments

### 10.1 Aim for 30 Lines, but Reject Mechanical Splits

30 lines is a readability signal, not a mechanical dividing line. Splitting "exactly to 30 lines" with meaningless helpers violates §15. When length grows, the options are: (a) split by responsibility into another function or class, (b) extract a helper **with a name that earns its keep**, (c) deduplicate via a type utility. If none apply, leave it long.

### 10.2 Aggregate Arguments Into an Options Object

If arguments exceed 3, aggregate into an `Options` type. Unlike Rust, TS has no named arguments; `foo(a, b, c, d, e)` invites positional mistakes at call sites. `Options` lets the caller pass by field name.

```ts
// NG
function foo(name: string, age: number, email: string, role: Role, ...) { ... }

// OK
type FooOptions = { name: string; age: number; email: string; role: Role; };
function foo(options: FooOptions) { ... }
```

"3" is not a hard threshold; rather, consider switching to `Options` the moment you want to add a 4th argument.

### 10.3 No Boolean Flag Arguments

Functions that read `flagged: boolean` and change behavior should be split. Either add a meaningful field to `Options`, split into two functions, or accept a discriminated union.

---

## 11. Imports

### 11.1 No `import *`

Namespace imports kill tree-shaking and hide what is actually used. Named imports only.

### 11.2 Types via `import type`

```ts
import type { User } from "./user";
import { parseUser } from "./user";
```

Separate runtime and type imports for tree-shaking and readability.

---

## 12. Documentation

### 12.1 TSDoc Required on Every Export

```ts
/**
 * Parses an email string. Returns `Email` (branded) on success,
 * `ParseError` on validation failure.
 */
export function parseEmail(input: string): Email | ParseError { ... }
```

### 12.2 Internal Comments Explain "Why," Not "What"

Comments that describe "what" are forbidden. Write "why this order," "why this boundary."

### 12.3 Do Not Cite Rule Numbers or Deleted History

- Forbidden: `// per docs/coding/ts.md §3.1` (section numbers in this document shift on reorganization)
- Forbidden: `// previously this used Foo` (referencing deleted code is meaningless)
- OK: `// behavior specified in docs/spec/tcml-format.md §"@clock"`

Write the reason in plain prose. Not by number.

### 12.4 No Japanese in Source

Source code is English-only. This covers:

- All comments — `//`, `/* */`, and TSDoc `/** */` — including "why" comments
- Identifiers, type names, file names

Narrow, pre-approved exceptions only:

- UI string literals shipped to Japanese users (e.g. `const PRIVACY_JA = "..."`)
- Test fixtures whose purpose is verifying non-ASCII handling

No new exception may be introduced by an agent or by a parent prompt. "TSDoc is OK in Japanese," "rationale comments are OK," "header comments are OK" — all false. If a comment must explain something, write it in English.

Documentation (`docs/**/*.md`, `README.md`) is Japanese as normal.

---

## 13. Tests

### 13.1 Tests in Separate Files

- Unit tests: `*.test.ts` in the same directory as the target
- Integration tests: `tests/` directory

Writing `describe(...)` inside a production file is forbidden.

### 13.2 Test Code Is Graded Leniently

Cosmetic details in tests (function length, local variable names, boilerplate) may be overlooked. However, **production-side safety rules** (`any` / `!` / `as X` / unvalidated `JSON.parse` / Japanese in source / suppression comments / widening `export` to expose internals to tests) apply to test code with full force.

---

## 14. Dead Code

Remove unused `export`, functions, and variables. Pushing past with eslint suppression is forbidden.

Exceptions:

1. Code used conditionally under specific build configurations
2. Helpers strictly required by tests

Even then, prefer build flags or conditional exports that avoid producing dead code in the first place.

---

## 15. Do Not Create Pointless Functions

A function costs a name, a signature, and a jump for the reader. The axis for judgment is **whether the name carries information the call site would lose**. Line count is not the test.

Justified by at least one of:

1. **Reuse** — used at two or more sites (or about to be)
2. **Naming** — the name carries meaning that inlining would lose. Valid even for one-line bodies
   - OK: `function isWeekend(day: Day): boolean { return day === "sat" || day === "sun"; }`
   - NG: `function addOne(n: number): number { return n + 1; }`
3. **Encapsulation** — enforces an invariant or hides a representation
4. **API contract** — required by an interface, callback, or public API

One-line wrappers meeting none of these are inlined.

---

## 16. Review Checklist

**Design**

- Duplication: does the logic already exist? Search before adding
- Lazy `export`: did you widen visibility instead of restructuring?
- Type not defined: are you passing raw `string` / `number` where a Branded Type or Discriminated Union is needed?
  - HTML built via `+` → DOM API or template tag
  - Raw SQL via `+` → prepared statement or query builder
  - Loose `status: string` → `status: "a" | "b" | "c"`
- Correctness: does it match the spec?

**Safety**

- Untrusted `NaN` / `Infinity` / negative values reaching numeric calculations?
- Strings reaching `innerHTML` / `eval` / `exec` / SQL unescaped?
- Has `any` crept in anywhere? (typical: after `JSON.parse`, `catch (e)`, external lib return values)
- Any `as` outside the §1.2 permitted list?

**Visibility**

- Any `export` without a real call site?
- Found one over-exposure → did you audit the whole file?

**Implementation**

- C-style / sloppy patterns (argument mutation, free functions, deep field chains, `==`, `for+push`) left in?
- Mutation scattered? `readonly` applied wherever possible?
- Forgot the `undefined` check on `arr[i]` under `noUncheckedIndexedAccess`?
- Reading a tuple newtype's `.value` from outside?
- Pointless wrapper (§15)?
- Error strategy in §2.3 consistent project-wide?

**Quality**

- TSDoc on every public item?
- Tests in separate files?
- "Why" comments in the right places, not citing rule numbers or deleted history?
- No Japanese in source?