# Boxroot for OCaml: fast movable GC roots

This library extends the OCaml foreign function interface with an
efficient and flexible GC rooting mechanism. See
<https://gitlab.com/ocaml-rust/ocaml-boxroot/>.

This crate exposes the raw functionality of the Boxroot library as
unsafe Rust functions. It is meant to be used by low-level libraries
to expose GC roots for the OCaml GC as smart pointers in Rust (see the
crate `ocaml-interop`).

## Running tests

The `link-ocaml-runtime-and-dummy-program` feature needs to be enabled when running tests:

    cargo test --features "link-ocaml-runtime-and-dummy-program"

## Feature flags

### `bundle-boxroot`

If this feature flag is not enabled (by default it is enabled), the
compilation of the C code that implements boxroot will be skipped,
and the user is responsible of linking boxroot into the final binary.

When this feature flag is enabled, the OCaml headers must be available
to be able to compile the boxroot C code.
