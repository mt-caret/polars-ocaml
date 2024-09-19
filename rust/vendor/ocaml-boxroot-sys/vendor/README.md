# Boxroot: fast movable roots for the OCaml-C and OCaml-Rust interfaces

This repository hosts a new root-registration API for the OCaml
garbage collector, compatible with OCaml 4 and OCaml 5. The new kind
of roots are called `boxroot` (boxed roots).

The official root-registration APIs of OCaml let users decide which
existing parts of memory should be considered as new roots by the
runtime. With boxroots, it is our allocator, not the user, that
decides where these roots are placed in memory. This extra flexibility
allows for a more efficient implementation.

We provide an implementation of this idea as a standalone C library
([boxroot/](boxroot/) in this repository), as a custom allocator using
OCaml's GC scanning hooks. Our prototype already shows promising
performance in benchmarks.

In addition to better performance, movable roots fit a common use-case
where Ocaml values are placed inside malloc'ed blocks and then
registered as global roots, for instance for insertion in C library
data structures. This pattern appears to be common. Our original
motivation for prototyping boxroots generalises that: it is to propose
an idiomatic manipulation of OCaml roots from Rust, similar to
`Box<T>` pointers.

We provide raw Rust bindings
([rust/ocaml-boxroot/](rust/ocaml-boxroot/) in this repository). These
bindings should not be used directly; they are used by libraries such
as [`ocaml-interop`](https://github.com/tizoc/ocaml-interop/) to
provide safe abstractions for the OCaml GC.

## Design

Functions to acquire, read, release and modify a `boxroot` are
provided as follows.

```c
boxroot boxroot_create(value);
value boxroot_get(boxroot);
value const * boxroot_get_ref(boxroot);
void boxroot_delete(boxroot);
int boxroot_modify(boxroot *, value);
```

These functions operate in constant time. (This can be compared to the
probabilistic logarithmic time offered by global roots.)

See [boxroot/boxroot.h](boxroot/boxroot.h) for API documentation.

## Benchmarks

To evaluate our experiment, we run various allocation-heavy
benchmarks.

### Implementations

The benchmarks compares various implementation of a cell containing a
single value with the same interface as boxroot (`create`, `get`,
`delete`, `modify`):

- `ocaml`: a pure OCaml implementation where `create` is the identity
  and nothing happens on deletion.
- `ocaml_ref`: a pure OCaml implementation using a mutable record, with
  deletion implemented by assigning `()` using Obj.magic.
- `gc`: a C implementation of the previous, using
  `caml_alloc_small(1,0)`,
- `boxroot`: a `boxroot` disguised as an immediate (reference
  implementation described further below),
- `naive`: like `boxroot`, but without taking advantage of the full
  expressiveness of boxroot (for the local roots benchmark below)
- `global`: a block allocated outside the OCaml heap (disguised as an
  immediate) containing a global root, and
- `generational`: idem, but using a generational
  global root.

Older experiments are not thread-safe (at least not efficiently so):

- `dll_boxroot`: a variant of `boxroot`, but using a simpler
  implementation with doubly-linked lists,
- `rem_boxroot`: a variant of `boxroot`, but using a different
  implementation using OCaml's remembered set.

The various implementations (except the `ocaml` one) have similar
memory representation, some on the OCaml heap and some outside of the
OCaml heap.

By selecting different implementations of Ref, we can evaluate the
overhead of root registration and scanning for various root
implementations, compared to non-rooting OCaml and C implementations,
along with other factors.

We can expect _a priori_ that `ocaml` is always faster, since it has
none of the overheads of other methods. This is meant as a baseline,
but it also shows a limitation of our benchmarks: `boxroot` is
intended for applications where values are _not_ easily reachable from
the OCaml heap and thus none of `ocaml`, `ocaml_ref` and `gc` are
available!

### Benchmark information

The figures below are obtained with OCaml 4.14 and OCaml 5.0
(development version), with CPU AMD Ryzen 5850U.

### Permutations of a list

The small program used in this benchmark computes the set of all
permutations of the list [0; ..; n-1], using a non-determinism monad
represented using (strict) lists. (This is an exponential way to
compute factorial(n) with lots of allocations.)

In our non-determinism monad, each list element goes through the Ref
module that boxes its underlying value, and may be implemented
(through C stubs) as an abstract block (not followed by the GC) whose
value is registered as a GC root.

This benchmark creates a lot of roots alive at the same time.

```
$ echo OCaml `ocamlc --version` && make run-perm_count TEST_MORE=2
OCaml 4.14.0
Benchmark: perm_count
---
boxroot: 1.71s
gc: 1.49s
ocaml: 1.20s
generational: 5.90s
ocaml_ref: 1.47s
dll_boxroot: 2.13s
rem_boxroot: 1.97s
global: 48.19s
```

This benchmark allocates 1860 boxroot pools, and performs 1077 minor
and 18 major collections. Roughly 22M boxroots are allocated.

We see that global roots add a large overhead, which is reduced by
using generational global roots. Boxroots outperform generational
global roots, and give slightly slower time than equivalent pure-GC
implementations (`ocaml_ref` and `gc`).

```
$ echo OCaml `ocamlc --version` && make run-perm_count TEST_MORE=2
OCaml 5.0.0+dev6-2022-07-21
Benchmark: perm_count
---
boxroot: 1.42s
gc: 1.62s
ocaml: 1.25s
generational: 5.43s
ocaml_ref: 1.69s
dll_boxroot: 1.69s
rem_boxroot: 1.55s
global: 49.74s
```

The OCaml 5.0 performance gives us a hint of considerations entering
into boxroot's performance. Here `boxroot` unexpectedly outperforms
`gc` and `ocaml_ref`. Two hypotheses can explain this speedup:
- `boxroot` puts less pressure on the GC. It indeed performs 14%
  fewer minor collections than `gc`.
- `boxroot` has good cache locality during root scanning. Indeed one
  major difference with OCaml 4.14 is the absence of prefetching
  during the major GC. The sensitivity to this effect is confirmed by
  running the benchmark with OCaml 4.12.0, which also lacks
  prefetching and shows `boxroot` also outperforming its pure-GC
  counterparts.

### Synthetic benchmark

In this benchmark, we allocate and deallocate values and roots
according to probabilities determined by parameters.

* `N=8`: log_2 of the number of minor generations
* `SMALL_ROOTS=10_000`: the number of small roots allocated (in the
  minor heap) per minor collection,
* `LARGE_ROOTS=20`: the number of large roots allocated (in the major
  heap) per minor collection,
* `SMALL_ROOT_PROMOTION_RATE=0.2`: the survival rate for small roots
  allocated in the current minor heap,
* `LARGE_ROOT_PROMOTION_RATE=1`: the survival rate for large roots
  allocated in the current minor heap,
* `ROOT_SURVIVAL_RATE=0.99`: the survival rate for roots that survived
  a first minor collection,
* `GC_PROMOTION_RATE=0.1`: promotion rate of GC-tracked values,
* `GC_SURVIVAL_RATE=0.5`: survival rate of GC-tracked values.

These settings favour the creation of a lot of roots, most of which
are short-lived. Roots that survive are few, but they are very
long-lived.

```
$ echo OCaml `ocamlc --version` && make run-synthetic TEST_MORE=2
OCaml 4.14.0
Benchmark: synthetic
---
boxroot: 6.19s
gc: 7.15s
ocaml: 6.21s
generational: 10.83s
ocaml_ref: 7.03s
dll_boxroot: 6.68s
rem_boxroot: 6.41s
global: 15.89s
```

This benchmark allocates 772 boxroot pools, and performs 2,619 minor
and 141 major collections. Roughly 16M roots are allocated.

`boxroot` again performs better than other root implementations, but
it is unexpected again that it outperforms `ocaml`, `ocaml_ref` and
`gc`. This is not well-understood.
- `gc` does actually fewer minor and major collection.
- Running with or without a prefetching GC shows comparable
  performance between `boxroot` and `ocaml`.

```
$ echo OCaml `ocamlc --version` && make run-synthetic TEST_MORE=2
OCaml 5.0.0+dev6-2022-07-21
Benchmark: synthetic
---
boxroot: 2.86s
gc: 2.98s
ocaml: 2.82s
generational: 4.80s
ocaml_ref: 2.99s
dll_boxroot: 3.04s
rem_boxroot: 3.01s
global: 6.08s
```

In multicore, this benchmark allocates 408 boxroot pools, and performs
2,615 minor and 55 major collections. 8.7M roots are allocated.
Therefore absolute values are not comparable between major OCaml
versions. The relative results are similar between OCaml 4.14 and
OCaml 5.0.

### Globroot benchmark

This benchmark is adapted from the OCaml testsuite. It exercises the
case where there are about 1024 concurrently-live roots, but only a
couple of young roots are created between two minor collections.

This benchmark tests the case where there are few concurrently-live
roots and little root creation and modification between two
collections. This benchmark does not perform any OCaml computations or
allocations (it forces collections to occur very often despite low GC
work). So the cost of root handling is magnified, it would normally be
amortized by OCaml computations.

```
$ echo OCaml `ocamlc --version` && make run-globroots TEST_MORE=2
OCaml 4.14.0
---
boxroot: 1.03s
gc: 1.35s
ocaml: 0.89s
generational: 1.14s
ocaml_ref: 1.27s
dll_boxroot: 0.99s
rem_boxroot: 0.95s
global: 1.24s
```

In this benchmark, there are about 67000 minor collections and 40000
major collections. 217k boxroots are allocated.

Since there are few root creations between collections, list-based and
remembered-set-based implementations are expected to perform well
(their scanning is quick). On the other hand, `boxroot` has to scan on
the order of a full memory pool at every minor collection even if
there are only a few young roots, for a pool size chosen large (16KB).
There used to be a noticeable overhead in this benchmark, but it has
been reduced with optimizations brought to scanning during minor
collection.

```
$ echo OCaml `ocamlc --version` && make run-globroots TEST_MORE=2
OCaml 5.0.0+dev6-2022-07-21
Benchmark: globroots
---
boxroot: 0.82s
gc: 1.01s
ocaml: 0.72s
generational: 0.92s
ocaml_ref: 1.02s
dll_boxroot: 0.84s
rem_boxroot: 0.78s
global: 1.34s
```


![Global roots benchmarks (OCaml 4.14)](global.svg)

![Global roots benchmarks (OCaml 5.0)](global5.svg)

### Local roots benchmark

We designed this benchmark to test the idea of replacing local
roots altogether by boxroots.

Currently, OCaml FFI code uses a "callee-roots" discipline where each
function has to locally "root" each OCaml value received as argument
or used as a temporary paramter, using the efficient `CAMLparam`,
`CAMLlocal`, `CAMLreturn` macros. These macros manage a shadow stack
containing pointers to the live roots.

Boxroots suggest a "caller-root" approach where callers would
package their OCaml values in boxroots, whose ownership is
passed to the callee. Creating boxroots is slower than
registering local roots, but the caller-root discipline can
avoid re-rooting each value when moving up and down the call
chain, so it is expected to have a performance advantage for
deep call chains.

This benchmark performs a (recursive) fixpoint computation on
OCaml floating-point value from C, with a parameter N that
decides the number of fixpoint iterations necessary, and thus
the length of the C-side call chain.

The local-roots version is as follows:

```c
int compare_val(value x, value y);

value local_fixpoint(value f, value x)
{
  CAMLparam2(f, x);
  CAMLlocal1(y);
  y = caml_callback(f,x);
  if (compare_val(x, y)) {
    CAMLreturn(y);
  } else {
    CAMLreturn(local_fixpoint(f, y));
  }
}
```
where `compare_val` compares the values of `x` and `y`, but introduces
local roots in order to simulate a more complex operation.

The boxroot version is as follows:

```c
value boxroot_fixpoint(value f, value x)
{
  boxroot f_root = boxroot_create(f);
  boxroot x_root = boxroot_create(x);
  boxroot y = boxroot_fixpoint_rooted(boxroot_get_ref(f), x_root);
  value v = boxroot_get(y);
  boxroot_delete(y);
  boxroot_delete(f_root);
  return v;
}

int compare_refs(value const *x, value const *y);

boxroot boxroot_fixpoint_rooted(value const *f, boxroot x)
{
  boxroot y = boxroot_create(caml_callback(*f, boxroot_get(x)));
  if (compare_refs(boxroot_get_ref(x), boxroot_get_ref(y))) {
    boxroot_delete(x);
    return y;
  } else {
    return boxroot_fixpoint_rooted(f, y);
  }
}
```
where `compare_refs` does the same work as `compare_val` but expects
its values already rooted.

The work is done by `boxroot_fixpoint_rooted`, but we need a
`boxroot_fixpoint` wrapper to go from the callee-roots convention
expected by OCaml `external` declarations to a caller-root convention.
(This wrapper also adds some overhead for small call depths.)

The `naive` test uses boxroots in a callee-roots discipline.


```
$ echo OCaml `ocamlc --version` && make run-local_roots TEST_MORE=2
OCaml 4.14.0
```

![Local roots benchmarks (OCaml 4.14)](local.svg)

We see that, in this test, despite the up-front cost of wrapping the
function, `boxroot`s are equivalent to or outperform OCaml's local
roots. More precisely, `boxroots` are slightly more expensive than
local roots when following the same callee-roots discipline, and the
caller-roots discipline offers huge saves in this benchmark. The saves
from the caller-roots discipline come from:
- introducing fewer roots,
- enabling recursion to be done via a tail call,
- enabling better code generation after inlining.

The results greatly depends on the test program and programming style.
For instance we did not take into account local roots optimisations
that fall outside of the documented syntactic rules and rely on expert
knowledge of their implementation, since one of our goal is to propose
an interface that can easily be made safe in Rust.

Our conclusions:
- Using boxroots is competitive with local roots.
- It can be beneficial if one leverages the added flexibility of
  boxroots.
- There could be specific scenarios where it is much more beneficial,
  for instance when traversing large OCaml structures from a foreign
  language, with many function calls.

Furthermore, we envision that with support from the OCaml compiler for
the caller-roots discipline, the wrapping responsible for initial
overhead could be made unnecessary.

```
$ echo OCaml `ocamlc --version` && make run-local_roots TEST_MORE=2
OCaml 5.0.0+dev6-2022-07-21
```

![Local roots benchmarks (OCaml 5.0)](local5.svg)

In multicore, we observe similar results. However, the overhead of C
calls appears to be higher, and (generational) global roots are much
more expensive since they are protected by a mutex.

We have measured the performance of two alternative implementations:
- _thread-unsafe_: assume that there is only one thread that never
  releases the domain lock, and thus avoid related checks in
  `boxroot_create` and `boxroot_delete`.
- _force remote_: the opposite, perform all deallocations as if they
  were done on a different domain, using the lock-free atomic
  deallocation path.

![Local roots benchmarks (impact of multicore support)](local5-2.svg)

Our thread-safe implementation of Boxroot for OCaml multicore is
slightly slower than a version that does not perform checks necessary
for thread-safety. The difference is likely lesser in other kinds of
situations where more time is spent in cache misses.

The implementation where all deallocations are done remotely is only
slightly slower than Boxroot (although with a much higher pool count
currently, due to the fact that remote deallocations are delayed until
the next garbage collection). However this single-threaded benchmark
does not let us see the costs of cache effects in realistic
multithreaded scenarios (cache misses and contention).

Our conclusions:
- The overhead of multithreading support is low enough to propose
  Boxroot as an all-purpose rooting machanism.
- The performance of cross-thread deallocation is likely very good,
  but we need better benchmarks to measure this.

## Implementation

We implemented a custom allocator that manages fairly standard
freelist-based memory pools, but we arrange to scan these pools
efficiently. In standard fashion, the pools are aligned in such a way
that the most significant bits can be used to identify the pool from
the address of their members. Since elements of the freelist are
guaranteed to point only inside the memory pool, and non-immediate
OCaml values are guaranteed to point only outside of the memory pool,
we can identify allocated slots as follows:

```
allocated(slot, pool) ≝ (pool != (slot & ~(1<<N - 2)))
```

N is a parameter determining the size of the pools. The bitmask is
chosen to preserve the least significant bit, so that immediate OCaml
values (those with lsb set) are correctly classified.

Scanning is set up by registering a root-scanning hook with the OCaml
GC, and done by traversing the pools linearly. An early-exit
optimisation when all roots have been found ensures that programs that
use few roots throughout the life of the program only pay for what
they use.

### Generational optimisation

The memory pools are managed in several rings, according to their
*class*. The class distinguishes pools according to OCaml generations,
as well as pools that are free (which need not be scanned). A pool is
*young* if it is allowed to contain pointers to the minor heap. During
minor collection, we only need to scan young pools. At the end of the
minor collection, the young pools, now guaranteed to no longer point
to any young value, are promoted into *old* pools.

We unconditionally allocate roots in young pools, to avoid testing at
allocation-time whether their initial value is young or old. This test
is better amortized if done from the collector for two reasons: in
some situations there may be many more roots allocated than living
through collections, and there is a way to make this test very
efficient when done in a tight loop. In addition, we want to inline
the fast paths of `boxroot_create` and `boxroot_delete`, so we gain in
reducing code size and avoiding branches that are not statically
predictible.

The rings are managed in such a manner that pools that are less than
half-full are rotated to the start of the ring. This ensures that it
is easy to find a pool for allocation. When the current pool is full,
if no young pool is available then we demote the first old pool into
a young pool, if it is less than half-full. (This pool contains major
roots, but it is harmless to scan them during minor collection.)
Otherwise we prefer to allocate a new pool.

Care is taken so that programs that do not allocate any root do not
pay any of the cost.

### Multicore implementation

In OCaml multicore, each domain has its own set of pool rings. Each
allocation is performed in the domain-local pools. As for
deallocations, each one is classified into: _local_, _remote domain_,
_purely remote_. (Improvements have been upstreamed for the release of
OCaml 5.0 to let us perform this classification efficiently.)

- Local deallocations are done on the same domain and while holding
  the domain lock. They are performed immediately without any
  synchronisation necessary.
- Remote domain deallocations are done from a different domain than
  the one that has allocated the boxroot. The typical use-case is
  sending OCaml values between threads, or foreign data structures
  containing such values. It is done while holding some domain lock,
  and thus we know that no interference with scanning is possible. It
  is pushed on an remote free list using an atomic exchange and an
  atomic increment. The remote free list is pushed back on top of the
  main free list at the start of scanning, which takes place during a
  stop-the-world section, when no other remote deallocation can take
  place.
- Purely remote deallocations are done without holding the domain
  lock. The typical use-case is the clean-up of foreign data
  structures that would store OCaml values while releasing the domain
  lock, which is rarer, so its performance is secondary. To avoid
  interference with scanning without making the latter very slow, each
  pool has a mutex that needs to be locked during purely remote
  deallocation. This mutex is also locked before scanning the pool. In
  all other aspects the purely remote deallocation is treated like a
  remote domain deallocation.

## Limitations

* This library has only been tested on Linux 64-bit, though it would
  be easy to port it to other platforms if it does not work right
  away. Please get in touch for any portability requirement.

* We have not yet written tests and benchmarks that exercise the
  multi-threading capability of Boxroot. All our benchmarks were
  single-threaded, although measuring an implementation which is
  thread-safe.

* Due to limitations of the GC hook interface, no work has been done
  to scan roots incrementally. Holding a (very!) large number of roots
  at the same time can negatively affect latency at the beginning of
  major GC cycles.
