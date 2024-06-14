/* SPDX-License-Identifier: MIT */
#define CAML_INTERNALS

#include "platform.h"
#include <stdlib.h>
#include <errno.h>

#if OCAML_MULTICORE

#include <caml/domain.h>
static_assert(Max_domains <= Num_domains,
              "OCaml is configured for a maximum number of domains greater than"
              " Boxroot's.");

#endif

pool * bxr_alloc_uninitialised_pool(size_t size)
{
  void *p = NULL;
  // TODO: portability?
  // Win32: p = _aligned_malloc(size, alignment);
  // int err = posix_memalign(&p, size, size);
  // assert(err != EINVAL);
  // if (err == ENOMEM) return NULL;
  p = aligned_alloc(size, size);
  assert(p != NULL);
  return p;
}

void bxr_free_pool(pool *p) {
    // Win32: _aligned_free(p);
    free(p);
}

bool bxr_initialize_mutex(pthread_mutex_t *mutex)
{
  return 0 == pthread_mutex_init(mutex, NULL);
}

void bxr_mutex_lock(pthread_mutex_t *mutex)
{
  pthread_mutex_lock(mutex);
}

void bxr_mutex_unlock(pthread_mutex_t *mutex)
{
  pthread_mutex_unlock(mutex);
}
