# Coding Conventions

## Fully Qualified Paths

Never use fully qualified paths inline. Always add a `use` import and refer to
the type by its short name. For example:

```rust
// correct
use std::cmp::Ordering;
// ...
.unwrap_or(Ordering::Equal)

// wrong
.unwrap_or(std::cmp::Ordering::Equal)
```

## Imports

Write one `use` item per line. Do not combine imports with `{}`.

```rust
// correct
use crate::generate::Depth;
use crate::generate::PopulationSize;

// wrong
use crate::generate::{Depth, PopulationSize};
```

## Variable Names

Avoid short (1–2 character) variable names and abbreviations. Use descriptive
names that make the role of the variable clear without needing to look at its
type or context. This applies to all bindings: `let`, closure parameters, match
arm bindings, function parameters, and struct field names.

Common abbreviations to avoid:

| Wrong | Right |
|-------|-------|
| `idx` | `index` |
| `ctx` | `context` |
| `cond` | `condition` or `predicate` |

## Types

Prefer specific, named newtype wrappers over bare primitives when a value has a
distinct meaning. For example, `PopulationSize(usize)` and `Depth(usize)` rather
than plain `usize`. Push these types as deep into the call stack as possible;
only unwrap with `.0` at the last moment where a primitive is unavoidable (e.g.
arithmetic, indexing, or standard-library calls).

## Behaviour in Custom Types

Push logic into custom types rather than operating on their contents from
outside. When you find yourself reaching into a type to perform a calculation,
ask whether that calculation belongs as a method or associated function on the
type itself.

Concrete examples from this codebase:
- `Depth::for_index(min, max, index)` instead of computing the range arithmetic
  at the call site.
- `PopulationSize::new_vec::<T>()` instead of calling `Vec::with_capacity`
  with an unwrapped value.
- `Population::random_index(rng)` instead of unwrapping the size and calling
  `rng.gen_range` at the call site.

A related signal: if you need `.0` outside the type's own module, that is a
strong hint that a method is missing.

## Variable Annotations

Always annotate variable bindings explicitly: `let x: X = something_else;`.
Omit the annotation when:
- the type is unwieldy to write out (e.g. deeply nested iterator chains or
  macro-generated types), or
- the type is unambiguously obvious from the right-hand side, such as a direct
  constructor call: `let depth = Depth(...)` needs no annotation because it
  cannot be anything other than `Depth`.
