/* SPDX-License-Identifier: MIT */
#ifndef OCAML_HOOKS_H
#define OCAML_HOOKS_H

#include <stdbool.h>
#include "platform.h"

#if OCAML_MULTICORE

#define bxr_domain_lock_held() (BXR_LIKELY(Caml_state_opt != NULL))

#else

/* true when the master lock is held, false otherwise */
extern _Thread_local bool bxr_thread_has_lock;

/* We need a way to detect concurrent mutations of
   [caml_enter/leave_blocking_section_hook]. They are only overwritten
   once at systhreads init (we assume that no piece of code in the
   OCaml ecosystem is as insane as the present one). If we see this
   happening, we place boxroot in safety mode (all allocations fail). */
#define bxr_domain_lock_held() (BXR_LIKELY(bxr_thread_has_lock))

#endif


#ifdef CAML_INTERNALS

#include <caml/mlvalues.h>
#include <caml/roots.h>
#include <caml/signals.h>
#include <caml/version.h>

#if OCAML_MULTICORE

#define CALL_GC_ACTION(action, data, v, p) action(data, v, p)
#define Add_to_ref_table(dom_st, p)                   \
  Ref_table_add(&dom_st->minor_tables->major_ref, p);

#else

#define CALL_GC_ACTION(action, data, v, p) do {       \
    action(v, p);                                     \
    (void)data;                                       \
  } while (0)
#define Add_to_ref_table(dom_st, p) add_to_ref_table(dom_st->ref_table, p)

#endif // OCAML_MULTICORE

typedef void (*bxr_scanning_callback) (scanning_action action,
                                       int only_young, void *data);

void bxr_setup_hooks(bxr_scanning_callback scanning,
                     caml_timing_hook domain_termination);

bool bxr_in_minor_collection();

#if !OCAML_MULTICORE

/* Used to regularly check that the hooks have not been overwritten.
   If they have, we place boxroot in safety mode. */
bool bxr_check_thread_hooks();

#endif // !OCAML_MULTICORE

#endif // CAML_INTERNALS

#endif // OCAML_HOOKS_H
