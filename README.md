# Color-Debug

Monkey-patches fmt machinery to colorize debug output.

![example](example.png)

## Limitations

This is of course very hacky, so anything may break in the future. Known limitations include:

- Struct and tuple names can only be colorized on nightly (requires `nightly` feature flag).
  - Specifically, derived `Debug` impls use internal shorthand methods which need to be hooked.
    If they were not, then manual and derived impls would look different.
- Derived unit structs/variants are not colorized, as they are just a write_str call.
- Field names are not colored when using nightly-only `field_with`.
- References to integer types are only colorized up to a certain depth, due to inlining.
