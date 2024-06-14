/* SPDX-License-Identifier: MIT */
/* {{{ Includes */

// This is emacs folding-mode

#include <assert.h>
#include <errno.h>
#include <limits.h>
#include <stdarg.h>
#include <stdalign.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#define CAML_NAME_SPACE
#define CAML_INTERNALS

#include "boxroot.h"
#include <caml/minor_gc.h>
#include <caml/major_gc.h>

#if defined(_POSIX_TIMERS) && defined(_POSIX_MONOTONIC_CLOCK)
#define POSIX_CLOCK
#include <time.h>
#endif

#include "ocaml_hooks.h"
#include "platform.h"

static_assert(!BXR_FORCE_REMOTE || BXR_MULTITHREAD,
              "invalid configuration");


caml_domain_state *caml_get_domain_state()
{
  return Caml_state;
}

/* }}} */

/* {{{ Data types */

enum {
  YOUNG = BXR_CLASS_YOUNG,
  OLD,
  UNTRACKED
};

struct bxr_private {
  /* _Atomic */ bxr_slot contents;
};

typedef struct {
  _Atomic(bxr_slot_ref) a_next;
  /* if non-empty, points to last cell */
  bxr_slot_ref end;
  /* length of the list */
  atomic_int a_alloc_count;
} atomic_free_list;

typedef struct pool {
  /* Each cell in `roots` has an owner who can access the cell.
     Unallocated cells are owned by the pool (thus by its domain). Who
     owns a boxroot owns its cell.

     In addition, the OCaml GC can access the cells concurrently. The
     OCaml GC assumes temporary ownership during stop-the-world
     sections, and while holding the mutex below.

     Consequently, access to the contents of `roots` is permitted for
     someone owning a cell either:
     - by holding _any_ domain lock, or
     - by holding the mutex below.

     The ownership discipline ensures that there are no concurrent
     mutations of the same cell coming from the mutator.

     To sum up, cells are protected by a combination of:
     - the user's ownership discipline,
     - the domain lock,
     - the pool mutex.

     Given that in order to dereference and modify a boxroot one needs
     a domain lock, the mutex is only needed by the mutator for the
     accesses during deallocations without holding any domain lock. */

  /* Free list, protected by domain lock. */
  bxr_free_list free_list;
  /* Owned by the pool ring. */
  struct pool *prev;
  struct pool *next;
  /* Note: `mutex` and `delayed_fl` are placed on their own cache
     line. Notably, together they exactly fit 8 words on Linux
     64-bit and this only wastes two padding words. */
  /* Delayed free list. Pushing is protected holding either of:
     - the pool mutex
     - a domain lock.
     Flushing is protected by holding both the pool mutex and all
     domain locks (or knowing no other thread owns a slot). */
  alignas(Cache_line_size) atomic_free_list delayed_fl;
  /* The pool mutex */
  mutex_t mutex;
  /* Allocated slots hold OCaml values. Unallocated slots hold a
     pointer to the next slot in the free list, or to the pool itself,
     denoting the empty free list. */
  bxr_slot roots[];
} pool;

#define POOL_CAPACITY ((int)((BXR_POOL_SIZE - sizeof(pool)) / sizeof(bxr_slot)))

static_assert(BXR_POOL_SIZE / sizeof(bxr_slot) <= INT_MAX, "pool size too large");
static_assert(POOL_CAPACITY >= 1, "pool size too small");
static_assert(offsetof(pool, free_list) == 0, "incorrect free_list offset");

/* }}} */

/* {{{ Globals */

/* Global pool rings. */
typedef struct {
  /* Pool of old values: contains only roots pointing to the major
     heap. Scanned at the start of major collection. */
  pool *old;
  /* Pool of young values: contains roots pointing to the major or to
     the minor heap. Scanned at the start of minor and major
     collection. */
  pool *young;
  /* Current pool. Ring of size 1. Scanned at the start of minor and
     major collection. */
  pool *current;
  /* Pools containing no root: not scanned.
     We could free these pools immediately, but this could lead to
     stuttering behavior for workloads that regularly come back to
     0 boxroots alive. Instead we wait for the next major root
     scanning to free empty pools. */
  pool *free;
} pool_rings;

/* Only accessed from one's own domain. Ownership requires the domain
   lock. */
static pool_rings *pools[Num_domains] = { NULL };

/* Holds the live pools of terminated domains until the next GC.
   Owned by orphan_mutex. */
static pool_rings orphan = { NULL, NULL, NULL, NULL };
static mutex_t orphan_mutex = BXR_MUTEX_INITIALIZER;

static bxr_free_list empty_fl = { (bxr_slot_ref)&empty_fl, NULL, -1, -1, UNTRACKED };

/* We cache the domain id for:
  - Fast detection of initialization (-1 if not initialized on this domain)
  - Lookup of current domain id fast and in parallel with other tests
*/
_Thread_local ptrdiff_t bxr_cached_dom_id = -1;

/* Only accessed from one's own domain. Ownership requires the domain
   lock. */
bxr_free_list *bxr_current_free_list[Num_domains + 1] =
  { &empty_fl, /* domain -1, always empty (trap for initialization) */
    &empty_fl, /* domain 0, accessed without initialization when
                  BXR_MULTITHREAD == 0 */
    /* NULL...*/ };

/* ownership required: domain */
static void set_current_fl(int dom_id, bxr_free_list *fl)
{
  DEBUGassert(dom_id >= 0 && dom_id < Num_domains);
  bxr_current_free_list[dom_id + 1] = fl;
}

/* ownership required: domain */
static void init_pool_rings(int dom_id)
{
  pool_rings *local = pools[dom_id];
  if (local == NULL) local = malloc(sizeof(pool_rings));
  if (local == NULL) return;
  local->old = NULL;
  local->young = NULL;
  local->current = NULL;
  local->free = NULL;
  set_current_fl(dom_id, &empty_fl);
  pools[dom_id] = local;
}

static struct {
  atomic_llong minor_collections;
  atomic_llong major_collections;
  atomic_llong total_create_young;
  atomic_llong total_create_old;
  atomic_llong total_create_slow;
  atomic_llong total_delete_young;
  atomic_llong total_delete_old;
  atomic_llong total_delete_slow;
  atomic_llong total_modify;
  atomic_llong total_modify_slow;
  atomic_llong total_gc_pool_rings;
  atomic_llong total_scanning_work_minor;
  atomic_llong total_scanning_work_major;
  atomic_llong total_minor_time;
  atomic_llong total_major_time;
  atomic_llong total_gc_pool_time;
  atomic_llong peak_minor_time;
  atomic_llong peak_major_time;
  atomic_llong total_alloced_pools;
  atomic_llong total_emptied_pools;
  atomic_llong total_freed_pools;
  atomic_llong live_pools; // number of tracked pools
  atomic_llong peak_pools; // max live pools at any time
  atomic_llong ring_operations; // Number of times p->next is mutated
  atomic_llong young_hit_gen; /* number of times a young value was encountered
                           during generic scanning (not minor collection) */
  atomic_llong young_hit_young; /* number of times a young value was encountered
                             during young scanning (minor collection) */
  atomic_llong get_pool_header; // number of times get_pool_header was called
  atomic_llong is_pool_member; // number of times is_pool_member was called
} stats;

/* }}} */

/* {{{ Tests in the hot path */

// hot path
/* ownership required: none */
static inline pool * get_pool_header(bxr_slot_ref s)
{
  if (DEBUG) incr(&stats.get_pool_header);
  return (pool *)Bxr_get_pool_header(s);
}

// Return true iff v shares the same msbs as p and is not an
// immediate.
// hot path
/* ownership required: none */
static inline bool is_pool_member(bxr_slot v, pool *p)
{
  if (DEBUG) incr(&stats.is_pool_member);
  return (uintptr_t)p == ((uintptr_t)v.as_slot_ref & ~((uintptr_t)BXR_POOL_SIZE - 2));
}

// hot path
static inline bool is_empty_free_list(bxr_slot_ref v, pool *p)
{
  return (v == (bxr_slot_ref)p);
}

/* }}} */

/* {{{ Ring operations */

/* ownership required: ring */
static inline void ring_link(pool *p, pool *q)
{
  p->next = q;
  q->prev = p;
  incr(&stats.ring_operations);
}

/* insert the ring [source] at the back of [*target]. */
/* ownership required: rings */
static inline void ring_push_back(pool *source, pool **target)
{
  if (source == NULL) return;
  DEBUGassert(source->prev == source && source->next == source);
  DEBUGassert(source != *target);
  if (*target == NULL) {
    *target = source;
  } else {
    DEBUGassert((*target)->free_list.class == source->free_list.class);
    pool *target_last = (*target)->prev;
    pool *source_last = source->prev;
    ring_link(target_last, source);
    ring_link(source_last, *target);
  }
}

// remove the first element from [*target] and return it
/* ownership required: rings */
static pool * ring_pop(pool **target)
{
  pool *front = *target;
  DEBUGassert(front != NULL);
  if (front->next == front) {
    *target = NULL;
  } else {
    *target = front->next;
    ring_link(front->prev, front->next);
  }
  ring_link(front, front);
  return front;
}

/* }}} */

/* {{{ Pool management */

/* the empty free-list for a pool p is denoted by a pointer to the
   pool itself (NULL could be a valid value for an element slot) */
/* ownership required: none */
static inline bxr_slot_ref empty_free_list(pool *p) { return (bxr_slot_ref)p; }

/* ownership required: pool */
static inline bool is_full_pool(pool *p)
{
  return is_empty_free_list(p->free_list.next, p);
}

/* ownership required: none */
static pool * get_empty_pool()
{
  long long live_pools = 1 + incr(&stats.live_pools);
  /* racy, but whatever */
  if (live_pools > stats.peak_pools) stats.peak_pools = live_pools;
  pool *p = bxr_alloc_uninitialised_pool(BXR_POOL_SIZE);
  if (p == NULL) return NULL;
  incr(&stats.total_alloced_pools);
  ring_link(p, p);
  p->free_list.next = p->roots;
  p->free_list.alloc_count = 0;
  p->free_list.end = &p->roots[POOL_CAPACITY - 1];
  p->free_list.domain_id = -1;
  p->free_list.class = UNTRACKED;
  store_relaxed(&p->delayed_fl.a_next, empty_free_list(p));
  store_relaxed(&p->delayed_fl.a_alloc_count, 0);
  p->delayed_fl.end = NULL;
  bxr_initialize_mutex(&p->mutex);
  /* We end the free_list with a dummy value which satisfies is_pool_member */
  p->roots[POOL_CAPACITY - 1].as_slot_ref = empty_free_list(p);
  for (bxr_slot_ref s = p->roots + POOL_CAPACITY - 2; s >= p->roots; --s) {
    s->as_slot_ref = s + 1;
  }
  return p;
}

/* ownership required: STW (or the current domain lock + knowledge
   that no other thread owns slots) */
static int anticipated_alloc_count(pool *p)
{
  return p->free_list.alloc_count + load_relaxed(&p->delayed_fl.a_alloc_count);
}

/* ownership required: STW (or the current domain lock + knowledge
   that no other thread owns slots)

   Returns 0 iff no element has been appended to the freelist.
*/
static int gc_pool(pool *p)
{
  int old_alloc_count = load_relaxed(&p->delayed_fl.a_alloc_count);
  if (0 == old_alloc_count) return 0;
  bxr_mutex_lock(&p->mutex);
  if (is_full_pool(p)) p->free_list.end = p->delayed_fl.end;
  p->free_list.alloc_count = anticipated_alloc_count(p);
  store_relaxed(&p->delayed_fl.a_alloc_count, 0);
  bxr_slot_ref list = p->free_list.next;
  p->free_list.next = load_relaxed(&p->delayed_fl.a_next);
  store_relaxed(&p->delayed_fl.a_next, empty_free_list(p));
  p->delayed_fl.end->as_slot_ref = list;
  bxr_mutex_unlock(&p->mutex);
  return old_alloc_count;
}

/* ownership required: ring */
static void free_pool_ring(pool **ring)
{
  while (*ring != NULL) {
    pool *p = ring_pop(ring);
    bxr_free_pool(p);
    incr(&stats.total_freed_pools);
  }
}

/* ownership required: rings */
static void free_pool_rings(pool_rings *ps)
{
  free_pool_ring(&ps->old);
  free_pool_ring(&ps->young);
  free_pool_ring(&ps->current);
  free_pool_ring(&ps->free);
}

/* }}} */

/* {{{ Pool class management */

/* ownership required: pool */
static inline bool is_not_too_full(pool *p)
{
  return p->free_list.alloc_count <= (int)(BXR_DEALLOC_THRESHOLD / sizeof(bxr_slot));
}

/* ownership required: domain, pool */
static void set_current_pool(int dom_id, pool *p)
{
  DEBUGassert(pools[dom_id]->current == NULL);
  if (p != NULL) {
    p->free_list.domain_id = dom_id;
    pools[dom_id]->current = p;
    p->free_list.class = YOUNG;
    set_current_fl(dom_id, &p->free_list);
  } else {
    set_current_fl(dom_id, &empty_fl);
  }
}

static void reclassify_pool(pool **source, int dom_id, int cl);

/* Move not-too-full pools to the front; move empty pools to the free
   ring. */
/* ownership required: domain, pool */
static void try_demote_pool(int dom_id, pool *p)
{
  DEBUGassert(p->free_list.class != UNTRACKED);
  pool_rings *remote = pools[dom_id];
  if (p == remote->current || !is_not_too_full(p)) return;
  int cl = (p->free_list.alloc_count == 0) ? UNTRACKED : p->free_list.class;
  /* If the pool is at the head of its ring, the new head must be
     recorded. */
  pool **source = (p == remote->old) ? &remote->old :
                  (p == remote->young) ? &remote->young : &p;
  reclassify_pool(source, dom_id, cl);
}

/* ownership required: ring */
static inline pool * pop_available(pool **target)
{
  /* When pools empty themselves enough, they are pushed to the front.
     When they fill up, they are pushed to the back. Thus, if the
     first one is full, then none of the next ones are empty
     enough. */
  if (*target == NULL || is_full_pool(*target)) return NULL;
  return ring_pop(target);
}

/* Find an available pool and set it as current. Return NULL if none
   was found and the allocation of a new one failed. */
/* ownership required: domain */
static pool * find_available_pool(int dom_id)
{
  pool_rings *local = pools[dom_id];
  pool *p = pop_available(&local->young);
  if (p == NULL && local->old != NULL && is_not_too_full(local->old))
    p = pop_available(&local->old);
  if (p == NULL) p = pop_available(&local->free);
  if (p == NULL) p = get_empty_pool();
  DEBUGassert(local->current == NULL);
  set_current_pool(dom_id, p);
  return p;
}

static void validate_all_pools(int dom_id);

/* move the head of [source] to the appropriate ring in domain
   [dom_id] determined by [class]. Not-too-full pools are pushed to
   the front. */
/* ownership required: ring, domain */
static void reclassify_pool(pool **source, int dom_id, int cl)
{
  DEBUGassert(*source != NULL);
  pool_rings *local = pools[dom_id];
  pool *p = ring_pop(source);
  p->free_list.domain_id = dom_id;
  pool **target = NULL;
  switch (cl) {
  case OLD: target = &local->old; break;
  case YOUNG: target = &local->young; break;
  case UNTRACKED:
    target = &local->free;
    incr(&stats.total_emptied_pools);
    decr(&stats.live_pools);
    break;
  }
  /* protected by domain lock */
  p->free_list.class = cl;
  ring_push_back(p, target);
  /* make p the new head of [*target] (rotate one step backwards) if
     it is not too full. */
  if (is_not_too_full(p)) *target = p;
}

/* ownership required: domain */
static void promote_young_pools(int dom_id)
{
  pool_rings *local = pools[dom_id];
  // Promote full pools
  while (local->young != NULL) {
    reclassify_pool(&local->young, dom_id, OLD);
  }
  // There is no current pool to promote. Ensure that a domain that
  // does not use any boxroot between two minor collections does not
  // pay the cost of scanning any pool.
  DEBUGassert(local->current == NULL);
}

/* }}} */

/* {{{ Allocation, deallocation */

/* Thread-safety: see documented constraints on the use of
   boxroot_setup and boxroot_teardown. */
static atomic_int status = BOXROOT_NOT_SETUP;

int boxroot_status()
{
  return load_relaxed(&status);
}

static bool setup();

static void try_gc_and_reclassify_one_pool_no_stw(pool **source, int dom_id);

// Set an available pool as current and allocate from it.
/* ownership required: current domain */
boxroot bxr_create_slow(value init)
{
  incr(&stats.total_create_slow);
  if (Caml_state_opt == NULL) { errno = EPERM; return NULL; }
  // We might be here because boxroot is not setup.
  if (0 == setup()) return NULL;
#if !OCAML_MULTICORE
  if (!bxr_domain_lock_held()) { errno = EPERM; return NULL; }
  if (!bxr_check_thread_hooks()) {
    status = BOXROOT_INVALID;
    return NULL;
  }
#endif
  int dom_id = Domain_id;
  /* Initialize pool rings on this domain */
  if (pools[dom_id] == NULL) init_pool_rings(dom_id);
  pool_rings *local = pools[dom_id];
  if (local == NULL) return NULL; /* ENOMEM */
  /* Initialization successful, now cache domain_id on this thread if
     not done. */
  if (bxr_cached_dom_id == -1) {
    bxr_cached_dom_id = dom_id;
    /* Perhaps we are here only because the domain id was not cached
       in the thread-local value. Try again. */
    return boxroot_create(init);
  } else {
    /* A thread cannot belong to two different OCaml domains at separate
       times (in particular registered threads always belong to domain
       0). */
    DEBUGassert(bxr_cached_dom_id == dom_id);
  }
  if (local->current != NULL) {
    /* Necessarily we are here because the pool is full */
    DEBUGassert(is_full_pool(local->current));
    /* We probably cannot garbage-collect the current pool, since it
       is highly unlikely that all the cells have been freed in the
       delayed_fl at this point. */
    reclassify_pool(&local->current, dom_id, YOUNG);
    /* Instead, whenever we fill a pool, we do enough work to
       garbage-collect any one young pool that may have been emptied
       remotely. This is quick since there are not many young pools.

       The goal is to avoid situations where remote deallocations fill
       pools faster than we garbage-collect them during STW (minor and
       major GC). So we do not look at old pools, because
       (heuristically) if it has time to survive a minor collection
       then it also has time to be collected during the subsequent
       minor collection.

       We can still have an excess of sparsely-populated pools due to
       remote deallocations. */
    try_gc_and_reclassify_one_pool_no_stw(&local->young, dom_id);
  }
  pool *p = find_available_pool(dom_id);
  if (p == NULL) return NULL; /* ENOMEM */
  DEBUGassert(!is_full_pool(p));
  /* Try again */
  return boxroot_create(init);
}

/* }}} */

/* {{{ Boxroot API implementation */

extern inline value boxroot_get(boxroot root);
extern inline value const * boxroot_get_ref(boxroot root);

/* ownership required: current domain */
void bxr_create_debug(value init)
{
  DEBUGassert(Caml_state_opt != NULL);
  if (Is_block(init) && Is_young(init)) incr(&stats.total_create_young);
  else incr(&stats.total_create_old);
}

extern inline boxroot boxroot_create(value init);

/* Needed to avoid linking error with Rust */
extern inline bool bxr_free_slot(bxr_free_list *fl, boxroot root);

/* ownership required: root, current domain */
void bxr_delete_debug(boxroot root)
{
  DEBUGassert(root != NULL);
  value v = boxroot_get(root);
  if (Is_block(v) && Is_young(v)) incr(&stats.total_delete_young);
  else incr(&stats.total_delete_old);
}

/* ownership required: root, any domain */
static void free_slot_atomic(pool *p, boxroot root)
{
  /* We have a domain lock, but not from the same domain as the pool.
     We perform a lock-free remote deallocation */
  /* Hey how do you avoid a CAS and the ABA problem? Well I only flush
     the delayed free list during stop-the-world sections or when the
     pool is empty! */
  bxr_slot_ref new_next = &root->contents;
  bxr_slot_ref old_next = atomic_exchange_explicit(&p->delayed_fl.a_next, new_next,
                                               memory_order_relaxed);
  new_next->as_slot_ref = old_next;
  if (BXR_UNLIKELY(is_empty_free_list(old_next, p)))
    p->delayed_fl.end = new_next;
  /* memory_order_release is needed here for flushing outside of STW
     sections (when the pool is empty). Otherwise memory_order_relaxed
     is enough. */
  decr_release(&p->delayed_fl.a_alloc_count);
}

/* ownership required: root, current domain */
void bxr_delete_slow(bxr_free_list *fl, boxroot root, bool remote)
{
  incr(&stats.total_delete_slow);
  pool *p = (pool *)fl;
  if (!remote) {
    /* We own the domain lock. Deallocation already done, but we
       passed a deallocation threshold. */
    try_demote_pool(p->free_list.domain_id, p);
  } else if (OCAML_MULTICORE && bxr_domain_lock_held()) {
    /* Remote, from another domain */
    free_slot_atomic(p, root);
  } else {
    /* No domain lock held */
    bxr_mutex_lock(&p->mutex);
    free_slot_atomic(p, root);
    bxr_mutex_unlock(&p->mutex);
  }
}

extern inline void boxroot_delete(boxroot root);

/* ownership required: root, current domain */
bool bxr_modify_slow(boxroot *root_ref, value new_value)
{
  incr(&stats.total_modify_slow);
  boxroot root = *root_ref;
  /* If the new value is not a young block, we can substitute. */
  if (!Is_block(new_value) || !Is_young(new_value)) {
    root->contents.as_value = new_value;
    return true;
  }
  /* Else, the pool is old and the value is young, so we need to
     reallocate */
  boxroot new = boxroot_create(new_value);
  if (BXR_UNLIKELY(new == NULL)) return false;
  *root_ref = new;
  boxroot_delete(root);
  return true;
}

void bxr_modify_debug(boxroot *rootp)
{
  DEBUGassert(*rootp);
  incr(&stats.total_modify);
}

extern inline bool boxroot_modify(boxroot *rootp, value new_value);

/* }}} */

/* {{{ Scanning */

/* ownership required: pool */
static void validate_pool(pool *pl)
{
  if (pl->free_list.next == NULL) {
    // an unintialised pool
    assert(pl->free_list.class == UNTRACKED);
    return;
  }
  // check free_list structure and length
  bxr_slot_ref curr = pl->free_list.next;
  int pos = 0;
  for (; !is_empty_free_list(curr, pl); curr = curr->as_slot_ref, pos++)
  {
    assert(pos < POOL_CAPACITY);
    assert(curr >= pl->roots && curr < pl->roots + POOL_CAPACITY);
  }
  assert(pos == POOL_CAPACITY - pl->free_list.alloc_count);
  // check count of allocated elements
  int count = 0;
  for(int i = 0; i < POOL_CAPACITY; i++) {
    bxr_slot s = pl->roots[i];
    --stats.is_pool_member;
    if (!is_pool_member(s, pl)) {
      value v = s.as_value;
      if (pl->free_list.class != YOUNG && Is_block(v)) assert(!Is_young(v));
      ++count;
    }
  }
  assert(count == anticipated_alloc_count(pl));
}

/* ownership required: pool */
static void validate_ring(pool **ring, int dom_id, int cl)
{
  pool *start_pool = *ring;
  if (start_pool == NULL) return;
  pool *p = start_pool;
  do {
    assert(p->free_list.domain_id == dom_id);
    assert(p->free_list.class == cl);
    validate_pool(p);
    assert(p->next != NULL);
    assert(p->next->prev == p);
    assert(p->prev != NULL);
    assert(p->prev->next == p);
    p = p->next;
  } while (p != start_pool);
}

/* ownership required: domain */
static void validate_all_pools(int dom_id)
{
  pool_rings *local = pools[dom_id];
  validate_ring(&local->old, dom_id, OLD);
  validate_ring(&local->young, dom_id, YOUNG);
  validate_ring(&local->current, dom_id, YOUNG);
  validate_ring(&local->free, dom_id, UNTRACKED);
}

static void gc_pool_rings(int dom_id);

/* ownership required: STW */
static void orphan_pools(int dom_id)
{
  pool_rings *local = pools[dom_id];
  if (local == NULL) return;
  gc_pool_rings(dom_id);
  bxr_mutex_lock(&orphan_mutex);
  /* Move active pools to the orphaned pools. TODO: NUMA awareness? */
  ring_push_back(local->old, &orphan.old);
  ring_push_back(local->young, &orphan.young);
  ring_push_back(local->current, &orphan.young);
  bxr_mutex_unlock(&orphan_mutex);
  /* Free the rest */
  free_pool_ring(&local->free);
  /* Reset local pools for later domains spawning with the same id */
  init_pool_rings(dom_id);
}

/* ownership required: domain */
static void adopt_orphaned_pools(int dom_id)
{
  bxr_mutex_lock(&orphan_mutex);
  while (orphan.old != NULL)
    reclassify_pool(&orphan.old, dom_id, OLD);
  while (orphan.young != NULL)
    reclassify_pool(&orphan.young, dom_id, YOUNG);
  bxr_mutex_unlock(&orphan_mutex);
}

/* ownership required: like gc_pool + ring */
static void try_gc_and_reclassify_pool(pool **source, int dom_id)
{
  pool *p = *source;
  if (gc_pool(p) != 0) {
    if (p->free_list.alloc_count == 0)
      reclassify_pool(source, dom_id, UNTRACKED);
    else if (is_not_too_full(p))
      reclassify_pool(source, dom_id, p->free_list.class);
  }
}

/* ownership required: ring, domain */
static void try_gc_and_reclassify_one_pool_no_stw(pool **source, int dom_id)
{
  pool *start = *source;
  pool *p = start;
  do {
    if (anticipated_alloc_count(p) == 0) {
      /* If the true alloc count is 0, then `delayed_alloc_count` is
         non-zero (otherwise the pool has already been reclassified
         into an untracked pool), and moreover we own all the slots
         (nobody can mutate the delayed_fl in parallel). Hence we have
         found a pool that we can usefully GC. */
      /* Synchronize with free_slot_atomic */
      atomic_thread_fence(memory_order_acquire);
      pool **new_source = (p == start) ? source : &p;
      try_gc_and_reclassify_pool(new_source, dom_id);
      return;
    }
    p = p->next;
  } while (p != start);
}

/* empty the delayed free lists in a ring and move the pools
   accordingly */
/* ownership required: STW */
static void gc_ring(pool **ring, int dom_id)
{
  if (!BXR_MULTITHREAD) return;
  pool *p = *ring;
  if (p == NULL) return;
  /* This is a bit convoluted because we do in-place GC-ing of the
     ring: we never move pools that did not need to be GCd. We
     distinguish two cases: if we are at the head of the ring or
     inside the tail. */
  /* GC the head of the ring while we are still at the head. */
  while (p == *ring) {
    pool *next = p->next;
    try_gc_and_reclassify_pool(ring, dom_id);
    if (p == next)
      /* There was only one pool left in the ring */
      return;
    p = next;
  }
  /* Now p != *ring, and things become easier */
  do {
    pool *next = p->next;
    try_gc_and_reclassify_pool(&p, dom_id);
    p = next;
  } while (p != *ring);
}

static long long time_counter(void);

/* empty the delayed free lists in the chosen pool rings and
   move the pools accordingly */
/* ownership required: STW */
static void gc_pool_rings(int dom_id)
{
  incr(&stats.total_gc_pool_rings);
  long long start = time_counter();
  pool_rings *local = pools[dom_id];
  // Heuristic: if a young pool has just been allocated, it is better
  // if it is the first one to be considered the next time a young
  // boxroot allocation takes place. (First it ends up last, so that
  // it ends up to the front after pool promotion.)
  if (local->current != NULL) {
    reclassify_pool(&local->current, dom_id, YOUNG);
    set_current_pool(dom_id, NULL);
  }
  gc_ring(&local->young, dom_id);
  gc_ring(&local->old, dom_id);
  long long duration = time_counter() - start;
  stats.total_gc_pool_time += duration;
}

// returns the amount of work done
/* ownership required: STW, pool mutex */
static int scan_pool_gen(scanning_action action, void *data, pool *pl)
{
  int allocs_to_find = anticipated_alloc_count(pl);
  int young_hit = 0;
  bxr_slot_ref current = pl->roots;
  while (allocs_to_find) {
    DEBUGassert(current < &pl->roots[POOL_CAPACITY]);
    // hot path
    bxr_slot s = *current;
    if (!is_pool_member(s, pl)) {
      --allocs_to_find;
      value v = s.as_value;
      if (DEBUG && Is_block(v) && Is_young(v)) ++young_hit;
      CALL_GC_ACTION(action, data, v, &current->as_value);
    }
    ++current;
  }
  stats.young_hit_gen += young_hit;
  return current - pl->roots;
}

/* Specialised version of [scan_pool_gen] when [only_young].

   Benchmark results for minor scanning:
   20% faster for young hits=95%
   20% faster for young hits=50% (random)
   90% faster for young_hit=10% (random)
   280% faster for young hits=0%
*/
/* ownership required: STW, pool mutex */
static int scan_pool_young(scanning_action action, void *data, pool *pl)
{
#if OCAML_MULTICORE
  /* If a <= b - 2 then
     a < x && x < b  <=>  x - a - 1 <= x - b - 2 (unsigned comparison)
  */
  uintnat young_start = (uintnat)caml_minor_heaps_start + 1;
  uintnat young_range = (uintnat)caml_minor_heaps_end - 1 - young_start;
#else
  uintnat young_start = (uintnat)Caml_state->young_start;
  uintnat young_range = (uintnat)Caml_state->young_end - young_start;
#endif
  bxr_slot_ref start = pl->roots;
  bxr_slot_ref end = start + POOL_CAPACITY;
  int young_hit = 0;
  bxr_slot_ref current;
  for (current = start; current < end; current++) {
    bxr_slot s = *current;
    value v = s.as_value;
    /* Optimise for branch prediction: if v falls within the young
       range, then it is likely that it is a block */
    if ((uintnat)v - young_start <= young_range
        && BXR_LIKELY(Is_block(v))) {
      ++young_hit;
      CALL_GC_ACTION(action, data, v, &current->as_value);
    }
  }
  stats.young_hit_young += young_hit;
  return current - start;
}

/* ownership required: STW */
static int scan_pool(scanning_action action, int only_young, void *data,
                     pool *pl)
{
  bxr_mutex_lock(&pl->mutex);
  int work = (only_young) ? scan_pool_young(action, data, pl)
                          : scan_pool_gen(action, data, pl);
  bxr_mutex_unlock(&pl->mutex);
  return work;
}

/* ownership required: STW */
static int scan_ring(scanning_action action, int only_young,
                     void *data, pool **ring)
{
  int work = 0;
  pool *start_pool = *ring;
  if (start_pool == NULL) return 0;
  pool *p = start_pool;
  do {
    work += scan_pool(action, only_young, data, p);
    p = p->next;
  } while (p != start_pool);
  return work;
}

/* ownership required: STW */
static int scan_pools(scanning_action action, int only_young,
                      void *data, int dom_id)
{
  pool_rings *local = pools[dom_id];
  int work = scan_ring(action, only_young, data, &local->young);
  if (!only_young) work += scan_ring(action, 0, data, &local->old);
  return work;
}

/* ownership required: STW */
static void scan_roots(scanning_action action, int only_young,
                       void *data, int dom_id)
{
  if (DEBUG) validate_all_pools(dom_id);
  /* First perform all the delayed deallocations. This also moves the
     current pool to the young pools. */
  gc_pool_rings(dom_id);
  /* The first domain arriving there will take ownership of the pools
     of terminated domains. */
  adopt_orphaned_pools(dom_id);
  int work = scan_pools(action, only_young, data, dom_id);
  if (bxr_in_minor_collection()) {
    promote_young_pools(dom_id);
  } else {
    free_pool_ring(&pools[dom_id]->free);
  }
  if (only_young) stats.total_scanning_work_minor += work;
  else stats.total_scanning_work_major += work;
  if (DEBUG) validate_all_pools(dom_id);
}

/* }}} */

/* {{{ Statistics */

static long long time_counter(void)
{
#if defined(POSIX_CLOCK)
  struct timespec t;
  clock_gettime(CLOCK_MONOTONIC, &t);
  return (long long)t.tv_sec * (long long)1000000000 + (long long)t.tv_nsec;
#else
  return 0;
#endif
}

// unit: 1=KiB, 2=MiB
static long long kib_of_pools(long long count, int unit)
{
  int log_per_pool = BXR_POOL_LOG_SIZE - unit * 10;
  if (log_per_pool >= 0) return count << log_per_pool;
  else return count >> -log_per_pool;
}

static double average(long long total, long long units)
{
  // round to nearest
  return ((double)total) / (double)units;
}

/* ownership required: none */
void boxroot_print_stats()
{
  printf("minor collections: %'lld\n"
         "major collections (and others): %'lld\n",
         stats.minor_collections,
         stats.major_collections);

  if (stats.total_alloced_pools == 0) return;

  printf("BXR_POOL_LOG_SIZE: %d (%'lld KiB, %'d roots/pool)\n"
         "DEBUG: %d\n"
         "OCAML_MULTICORE: %d\n"
         "BXR_MULTITHREAD: %d\n"
         "BXR_FORCE_REMOTE: %d\n",
         (int)BXR_POOL_LOG_SIZE, kib_of_pools(1, 1), (int)POOL_CAPACITY,
         (int)DEBUG, (int)OCAML_MULTICORE,
         (int)BXR_MULTITHREAD, (int)BXR_FORCE_REMOTE);

  printf("total allocated pools: %'lld (%'lld MiB)\n"
         "peak allocated pools: %'lld (%'lld MiB)\n"
         "total emptied pools: %'lld (%'lld MiB)\n"
         "total freed pools: %'lld (%'lld MiB)\n",
         stats.total_alloced_pools,
         kib_of_pools(stats.total_alloced_pools, 2),
         stats.peak_pools,
         kib_of_pools(stats.peak_pools, 2),
         stats.total_emptied_pools,
         kib_of_pools(stats.total_emptied_pools, 2),
         stats.total_freed_pools,
         kib_of_pools(stats.total_freed_pools, 2));

  double scanning_work_minor =
    average(stats.total_scanning_work_minor, stats.minor_collections);
  double scanning_work_major =
    average(stats.total_scanning_work_major, stats.major_collections);
  long long total_scanning_work =
    stats.total_scanning_work_minor + stats.total_scanning_work_major;
#if DEBUG
  double young_hits_gen_pct =
    average(stats.young_hit_gen * 100, stats.total_scanning_work_major);
#endif
  double young_hits_young_pct =
    average(stats.young_hit_young * 100, stats.total_scanning_work_minor);

  printf("work per minor: %'.0f\n"
         "work per major: %'.0f\n"
         "total scanning work: %'lld (%'lld minor, %'lld major)\n"
#if DEBUG
         "young hits (non-minor collection): %.2f%%\n"
#endif
         "young hits (minor collection): %.2f%%\n",
         scanning_work_minor,
         scanning_work_major,
         total_scanning_work, stats.total_scanning_work_minor, stats.total_scanning_work_major,
#if DEBUG
         young_hits_gen_pct,
#endif
         young_hits_young_pct);

#if defined(POSIX_CLOCK)
  double time_per_minor =
    average(stats.total_minor_time, stats.minor_collections) / 1000;
  double time_per_major =
    average(stats.total_major_time, stats.major_collections) / 1000;
  double time_per_gc_pool_rings =
    average(stats.total_gc_pool_time, stats.total_gc_pool_rings) / 1000;

  printf("average time per minor: %'.3fµs\n"
         "average time per major: %'.3fµs\n"
         "peak time per minor: %'.3fµs\n"
         "peak time per major: %'.3fµs\n"
         "average time per gc_pool_rings: %'.3fµs\n",
         time_per_minor,
         time_per_major,
         ((double)stats.peak_minor_time) / 1000,
         ((double)stats.peak_major_time) / 1000,
         time_per_gc_pool_rings);
#endif

  double ring_operations_per_pool =
    average(stats.ring_operations, stats.total_alloced_pools);

  printf("total boxroot_create_slow: %'lld\n"
         "total boxroot_delete_slow: %'lld\n"
         "total boxroot_modify_slow: %'lld\n"
         "total ring operations: %'lld\n"
         "ring operations per pool: %.2f\n"
         "total gc_pool_rings: %'lld\n",
         stats.total_create_slow,
         stats.total_delete_slow,
         stats.total_modify_slow,
         stats.ring_operations,
         ring_operations_per_pool,
         stats.total_gc_pool_rings);

#if DEBUG
  long long total_create = stats.total_create_young + stats.total_create_old;
  long long total_delete = stats.total_delete_young + stats.total_delete_old;
  double create_young_pct =
    average(stats.total_create_young * 100, total_create);
  double delete_young_pct =
    average(stats.total_delete_young * 100, total_delete);

  printf("total created: %'lld (%.2f%% young)\n"
         "total deleted: %'lld (%.2f%% young)\n"
         "total modified: %'lld\n",
         total_create, create_young_pct,
         total_delete, delete_young_pct,
         stats.total_modify);

  printf("get_pool_header: %'lld\n"
         "is_pool_member: %'lld\n",
         stats.get_pool_header,
         stats.is_pool_member);
#endif
}

/* }}} */

/* {{{ Hook setup */

/* ownership required: STW */
static void scanning_callback(scanning_action action, int only_young,
                              void *data)
{
  if (boxroot_status() == BOXROOT_NOT_SETUP
      || boxroot_status() == BOXROOT_TORE_DOWN) return;
  bool in_minor_collection = bxr_in_minor_collection();
  if (in_minor_collection) incr(&stats.minor_collections);
  else incr(&stats.major_collections);
  int dom_id = Domain_id;
  if (pools[dom_id] == NULL) return; /* synchronised by domain lock */
#if !OCAML_MULTICORE
  if (!bxr_check_thread_hooks()) status = BOXROOT_INVALID;
#endif
  long long start = time_counter();
  scan_roots(action, only_young, data, dom_id);
  long long duration = time_counter() - start;
  atomic_llong *total = in_minor_collection ? &stats.total_minor_time : &stats.total_major_time;
  atomic_llong *peak = in_minor_collection ? &stats.peak_minor_time : &stats.peak_major_time;
  *total += duration;
  if (duration > *peak) *peak = duration; // racy, but whatever
}

/* Handle orphaning of domain-local pools */
/* ownership required: current domain */
static void domain_termination_callback()
{
  DEBUGassert(OCAML_MULTICORE == 1);
  int dom_id = Domain_id;
  orphan_pools(dom_id);
}

/* Used for initialization/teardown */
static mutex_t init_mutex = BXR_MUTEX_INITIALIZER;

/* ownership required: current domain */
static bool setup()
{
  if (boxroot_status() == BOXROOT_RUNNING) return true;
  bool res = true;
  bxr_mutex_lock(&init_mutex);
  if (status != BOXROOT_NOT_SETUP) {
    res = (status == BOXROOT_RUNNING);
    goto out;
  }
  bxr_setup_hooks(&scanning_callback, &domain_termination_callback);
  // we are done
  status = BOXROOT_RUNNING;
  // fall through
 out:
  bxr_mutex_unlock(&init_mutex);
  return res;
}

/* With OCaml < 5.0, runtime lock detection must be setup before any
   thread is created, otherwise it does not work. It must also be
   installed after threads initialization. In addition, [setup] must
   be called if one cannot guarantee that the first boxroot allocation
   runs while holding the lock. */
bool boxroot_setup() { return setup(); }

/* We are sole owner of the pools at this point, no need for
   locking. */
void boxroot_teardown()
{
  bxr_mutex_lock(&init_mutex);
  if (status != BOXROOT_RUNNING) goto out;
  status = BOXROOT_TORE_DOWN;
  for (int i = 0; i < Num_domains; i++) {
    pool_rings *ps = pools[i];
    if (ps == NULL) continue;
    free_pool_rings(ps);
    free(ps);
    set_current_fl(i, &empty_fl);
  }
  free_pool_rings(&orphan);
  // fall through
 out:
  bxr_mutex_unlock(&init_mutex);
}

/* }}} */

/* {{{ */
/* }}} */
