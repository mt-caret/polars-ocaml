/* SPDX-License-Identifier: MIT */
#ifndef BOXROOT_PLATFORM_H
#define BOXROOT_PLATFORM_H

#define CAML_NAME_SPACE

#include <stdbool.h>
#include <caml/config.h>
#include <caml/version.h>

typedef intnat value;

#if defined(__GNUC__)
#define BXR_LIKELY(a) __builtin_expect(!!(a),1)
#define BXR_UNLIKELY(a) __builtin_expect(!!(a),0)
#else
#define BXR_LIKELY(a) (a)
#define BXR_UNLIKELY(a) (a)
#endif

#if (OCAML_VERSION >= 50000) && (!defined BUILD_RS || defined OCAML_VERSION_5)
#include <caml/domain_state.h>
#define OCAML_MULTICORE true
#else
#define Caml_state_opt Caml_state
#define OCAML_MULTICORE false
#endif

#ifdef CAML_INTERNALS

#include <assert.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stddef.h>
#include <caml/mlvalues.h>
#include <caml/minor_gc.h>
#include <caml/roots.h>

#if OCAML_MULTICORE

/* We currently rely on OCaml 5.0 having a max number of domains; this
   is checked for consistency. */
#define Num_domains 128
#define Domain_id (Caml_state->id)

#else

#define Num_domains 1
#define Domain_id 0

#endif // OCAML_MULTICORE

#define Cache_line_size 64
//#define Cache_line_size 128 /* Apple M1, Intel spatial prefetcher since Sandy Bridge */

#define load_relaxed(a) (atomic_load_explicit((a), memory_order_relaxed))
#define load_acquire(a) (atomic_load_explicit((a), memory_order_acquire))
#define store_relaxed(a, n) (atomic_store_explicit((a), (n), memory_order_relaxed))
#define incr(a) (atomic_fetch_add_explicit((a), 1, memory_order_relaxed))
#define decr(a) (atomic_fetch_add_explicit((a), -1, memory_order_relaxed))
#define decr_release(a) (atomic_fetch_add_explicit((a), -1, memory_order_release))

typedef pthread_mutex_t mutex_t;
#define BXR_MUTEX_INITIALIZER PTHREAD_MUTEX_INITIALIZER;

bool bxr_initialize_mutex(mutex_t *mutex);
void bxr_mutex_lock(mutex_t *mutex);
void bxr_mutex_unlock(mutex_t *mutex);

/* Check integrity of pool structure after each scan, and print
   additional statistics? (slow)
   This can be enabled by passing BOXROOT_DEBUG=1 as argument. */
#if defined(BOXROOT_DEBUG) && (BOXROOT_DEBUG == 1)
#define DEBUG true
#define DEBUGassert(x) assert(x)
#else
#define DEBUG false
#if defined(__GNUC__)
#define DEBUGassert(x) do { if (!(x)) { __builtin_unreachable(); } } while (0)
#else
#define DEBUGassert(x) ((void)0)
#endif
#endif

typedef struct pool pool;

pool* bxr_alloc_uninitialised_pool(size_t size);
void bxr_free_pool(pool *p);

#endif // CAML_INTERNALS

#endif // BOXROOT_PLATFORM_H
