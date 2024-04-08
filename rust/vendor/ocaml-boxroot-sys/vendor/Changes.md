ocaml-boxroot 0.3.1
===================

### General changes

- Introduce `boxroot_setup_systhreads()`, which must be called when
  using OCaml 4 and threads to avoid a bug. It must be called between
  OCaml startup and first thread creation.
  (Guillaume Munch-Maccagnoni, report and suggestion by Gabriel
  Scherer)


ocaml-boxroot 0.3.0
===================

### General changes

- Add support for OCaml 5.0 (multicore).
  (Guillaume Munch-Maccagnoni, review by Gabriel Scherer)

- Avoid locking a mutex in the fast paths. The option
  `BOXROOT_USE_MUTEX` is removed and the implementation is now
  thread-safe by default.
  (Guillaume Munch-Maccagnoni, review by Gabriel Scherer)

- Boxroot setup is now automatic and `boxroot_setup` is obsolete.
  (Guillaume Munch-Maccagnoni, review by Gabriel Scherer)

- Clarify license (MIT license).
  (Guillaume Munch-Maccagnoni, review by Gabriel Scherer)

- Better error reporting and analysis for API authors
  (`boxroot_status`). `boxroot_create` and `boxroot_modify` now detect
  and safely fail when called without holding the domain lock
  (signature change for `boxroot_modify`).
  (Guillaume Munch-Maccagnoni)

### Internal changes

- Detection of master lock for OCaml 4. Note usage constraints with
  systhreads for OCaml 4 (see documentation of `boxroot_status` in
  `boxroot.h`).
  (Guillaume Munch-Maccagnoni, review by Gabriel Scherer)

- Benchmark improvements.
  (Gabriel Scherer and Guillaume Munch-Maccagnoni)

- Per-domain caching for OCaml multicore. There is no longer a global
  lock for multicore.
  (Guillaume Munch-Maccagnoni)

- Various performance improvements under OCaml 4 and OCaml 5.
  Deallocation is almost-always lock-free.
  (Guillaume Munch-Maccagnoni)

### Experiments

- Simple implementation with a doubly-linked list
  (Gabriel Scherer, review by Jacques-Henri Jourdan and
   Guillaume Munch-Maccagnoni)

- Implementation using the remembered set and a per-pool young
  freelist.
  (Gabriel Scherer, following an idea of Stephen Dolan, review
   by Guillaume Munch-Maccagnoni)

- Optimizing for inlining.
  (Guillaume Munch-Maccagnoni, review by Gabriel Scherer)

### Packaging

- API update for ocaml-boxroot-sys (breaking change).
  (Guillaume Munch-Maccagnoni, review by Zach Shipko)

- Minor improvements.
  (Guillaume Munch-Maccagnoni)

- Remove `without-ocamlopt` feature flag from the Rust crate and add
  `bundle-boxroot`.
  (Bruno Deferrari, review by Guillaume Munch-Maccagnoni)

- Declare `package.links` value in Rust crate.
  (Bruno Deferrari, review by Guillaume Munch-Maccagnoni)

- ocaml-boxroot-sys no longer links to the std library (`[no_std]`).
  Note though that ocaml-boxroot still relies on a system allocator
  (`posix_memalign`).
  (Guillaume Munch-Maccagnoni)


ocaml-boxroot 0.2
=================

### General changes

- Thread-safety using a global lock.
  (Bruno Deferrari)

### Internal changes

- Minor simplifications and performance improvements to the allocator
  and to the benchmarks.
  (Gabriel Scherer and Guillaume Munch-Maccagnoni)

### Packaging

- Add `without-ocamlopt` feature flag for compiling the Rust crate
  without an OCaml install requirement.
  (Bruno Deferrari)


ocaml-boxroot 0.1
=================

- First numbered prototype & experimentation.
  (Bruno Deferrari, Guillaume Munch-Maccagnoni, Gabriel Scherer)

### Packaging

- First version published with the Rust crate ocaml-boxroot-sys.
  (Bruno Deferrari)
