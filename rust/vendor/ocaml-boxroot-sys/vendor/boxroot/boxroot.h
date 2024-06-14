/* SPDX-License-Identifier: MIT */
#ifndef BOXROOT_H
#define BOXROOT_H

#include <stdbool.h>
#include "ocaml_hooks.h"
#include "platform.h"

/* `boxroot`s follow an ownership discipline. */
typedef struct bxr_private* boxroot;

/* `boxroot_create(v)` allocates a new boxroot initialised to the
   value `v`. This value will be considered as a root by the OCaml GC
   as long as the boxroot lives or until it is modified. The OCaml
   domain lock must be held before calling `boxroot_create`.

   A return value of `NULL` indicates a failure of allocation or
   initialization of Boxroot (see `boxroot_status`). */
inline boxroot boxroot_create(value);

/* `boxroot_get(r)` returns the contained value, subject to the usual
   discipline for non-rooted values. `boxroot_get_ref(r)` returns a
   pointer to a memory cell containing the value kept alive by `r`,
   that gets updated whenever its block is moved by the OCaml GC. The
   pointer becomes invalid after any call to `boxroot_delete(r)` or
   `boxroot_modify(&r,v)`. The argument must be non-null.

   The OCaml domain lock must be held before calling `boxroot_get` or
   before deferencing the result of `boxroot_get_ref`.
*/
inline value boxroot_get(boxroot r) { return *(value *)r; }
inline value const * boxroot_get_ref(boxroot r) { return (value *)r; }

/* `boxroot_delete(r)` deallocates the boxroot `r`. The value is no
   longer considered as a root by the OCaml GC. The argument must be
   non-null. (One does not need to hold the OCaml domain lock before
   calling `boxroot_delete`.)*/
inline void boxroot_delete(boxroot);

/* `boxroot_modify(&r,v)` changes the value kept alive by the boxroot
   `r` to `v`. It is essentially equivalent to the following:
   ```
   boxroot_delete(r);
   r = boxroot_create(v);
   ```
   However, `boxroot_modify` is more efficient: it avoids reallocating
   the boxroot if possible. The reallocation, if needed, occurs at
   most once between two minor collections.

   The OCaml domain lock must be held before calling `boxroot_modify`.

   A return value of `false` indicates that the modification could not
   take place due to a failure of reallocation (see `boxroot_status`).
*/
inline bool boxroot_modify(boxroot *, value);

/* With OCaml 4 and when using threads, `boxroot_setup_systhreads()`
   must be called after OCaml startup, but before creating any thread
   and before allocating any boxroot. Obsolete under OCaml ≥ 5.0. */
static inline void boxroot_setup_systhreads();

/* `boxroot_teardown()` releases all the resources of Boxroot. None of
   the function above must be called after this. `boxroot_teardown`
   can only be called after OCaml shuts down. */
void boxroot_teardown();

/* For API authors, `boxroot_status()` shows the cause of an
   allocation failure:

   - Permanent failures:
       - `BOXROOT_TORE_DOWN`: `boxroot_teardown` has been called.
       - `BOXROOT_INVALID`: Boxroot is in an an invalid state when
         using OCaml 4 and threads. Results from not properly calling
         `boxroot_setup_systhreads()` between OCaml startup and
         creating any thread.

   - Transient failures (`BOXROOT_RUNNING`), check `errno`:
       - `errno == EPERM`: you tried calling `boxroot_create` or
         `boxroot_modify` without holding the domain lock.
       - `errno == ENOMEM`: allocation failure of the backing store. */
enum {
  BOXROOT_NOT_SETUP,
  BOXROOT_RUNNING,
  BOXROOT_TORE_DOWN,
  BOXROOT_INVALID
};
int boxroot_status();

/* Show some statistics on the standard output. */
void boxroot_print_stats();

/* Obsolete. */
bool boxroot_setup();


/* ================================================================= */


/* Private implementation. All identifiers starting with bxr_ are private. */

typedef /* _Atomic */ union bxr_slot *bxr_slot_ref;

typedef union bxr_slot {
  bxr_slot_ref as_slot_ref;
  value as_value;
} bxr_slot;

typedef struct bxr_free_list {
  bxr_slot_ref next;
  /* if non-empty, points to last cell */
  bxr_slot_ref end;
  /* length of the list */
  int alloc_count;
  int domain_id;
  /* kept in sync with its location in the pool rings. */
  int class;
} bxr_free_list;

#define BXR_CLASS_YOUNG 0

extern _Thread_local ptrdiff_t bxr_cached_dom_id;
extern bxr_free_list *bxr_current_free_list[/*Num_domains + 1*/];

void bxr_create_debug(value v);
boxroot bxr_create_slow(value v);

/* Test the overheads of multithreading (systhreads and multicore).
   Purely for experimental purposes. Otherwise should always be true. */
#define BXR_MULTITHREAD true
/* Make every deallocation a remote deallocation. For testing purposes
   only. Otherwise should always be 0. */
#define BXR_FORCE_REMOTE false

inline boxroot boxroot_create(value init)
{
#if defined(BOXROOT_DEBUG) && (BOXROOT_DEBUG == 1)
  bxr_create_debug(init);
#endif
  /* Find current free_list. Synchronized by domain lock. */
  ptrdiff_t dom_id =
    (OCAML_MULTICORE && BXR_MULTITHREAD) ? bxr_cached_dom_id : 0;
  bxr_free_list *fl = bxr_current_free_list[dom_id + 1];
  bxr_slot_ref new_root = fl->next;
  if (BXR_UNLIKELY(!bxr_domain_lock_held())
      || BXR_UNLIKELY(new_root == (bxr_slot_ref)fl))
    return bxr_create_slow(init);
  fl->next = new_root->as_slot_ref;
  fl->alloc_count++;
  new_root->as_value = init;
  return (boxroot)new_root;
}

/* Log of the size of the pools (12 = 4KB, an OS page).
   Recommended: 14. */
#define BXR_POOL_LOG_SIZE 14
#define BXR_POOL_SIZE ((size_t)1 << BXR_POOL_LOG_SIZE)
/* Every DEALLOC_THRESHOLD deallocations, make a pool available for
   allocation or demotion into a young pool, or reclassify it as an
   empty pool if empty. Change this with benchmarks in hand. Must be a
   power of 2. */
#define BXR_DEALLOC_THRESHOLD ((int)BXR_POOL_SIZE / 2)

#define Bxr_get_pool_header(s)                                      \
  ((bxr_free_list *)((uintptr_t)(s) & ~((uintptr_t)BXR_POOL_SIZE - 1)))

inline bool bxr_free_slot(bxr_free_list *fl, boxroot root)
{
  /* We have the lock of the domain that owns the pool. */
  bxr_slot_ref s = (bxr_slot_ref)root;
  bxr_slot_ref next = fl->next;
  s->as_slot_ref = next;
  if (BXR_MULTITHREAD && BXR_UNLIKELY(next == (bxr_slot_ref)fl))
    fl->end = s;
  fl->next = s;
  int alloc_count = --fl->alloc_count;
  return (alloc_count & (BXR_DEALLOC_THRESHOLD - 1)) == 0;
}

void bxr_delete_debug(boxroot root);
void bxr_delete_slow(bxr_free_list *fl, boxroot root, bool remote);

inline void boxroot_delete(boxroot root)
{
#if defined(BOXROOT_DEBUG) && (BOXROOT_DEBUG == 1)
  bxr_delete_debug(root);
#endif
  bxr_free_list *fl = Bxr_get_pool_header(root);
  bool remote_dom_id =
    OCAML_MULTICORE ? fl->domain_id != bxr_cached_dom_id : false;
  bool remote =
    BXR_FORCE_REMOTE
    || (BXR_MULTITHREAD
        && (BXR_UNLIKELY(remote_dom_id) || !bxr_domain_lock_held()));
  if (remote || BXR_UNLIKELY(bxr_free_slot(fl, root)))
    /* remote deallocation or deallocation threshold */
    bxr_delete_slow(fl, root, remote);
}

void bxr_modify_debug(boxroot *rootp);
bool bxr_modify_slow(boxroot *rootp, value new_value);

inline bool boxroot_modify(boxroot *rootp, value new_value)
{
#if defined(BOXROOT_DEBUG) && (BOXROOT_DEBUG == 1)
  bxr_modify_debug(rootp);
#endif
  if (BXR_UNLIKELY(!bxr_domain_lock_held())) return 0;
  bxr_slot_ref s = (bxr_slot_ref)*rootp;
  bxr_free_list *fl = Bxr_get_pool_header(s);
  if (BXR_LIKELY(fl->class == BXR_CLASS_YOUNG)) {
    s->as_value = new_value;
    return 1;
  } else {
    /* We might need to reallocate, but this reallocation happens at
       most once between two minor collections. */
    return bxr_modify_slow(rootp, new_value);
  }
}

#if !OCAML_MULTICORE

/* From ocaml/otherlibs/systhreads/st_stubs.c */
value caml_thread_initialize(value unit);

/* With systhreads, `boxroot_setup` must be called after
   `caml_thread_initialize` but before any thread is created. Using
   `static inline` to avoid causing linking errors with this
   definition when systhreads is not linked.*/
static inline void boxroot_setup_systhreads()
{
  caml_thread_initialize(1);
  boxroot_setup();
}

#else

static inline void boxroot_setup_systhreads() {}

#endif

#endif // BOXROOT_H
