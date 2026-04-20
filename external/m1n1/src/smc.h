/* SPDX-License-Identifier: MIT */

#ifndef SMC_H
#define SMC_H

#include "asc.h"
#include "rtkit.h"
#include "types.h"

typedef struct smc_dev smc_dev_t;

int smc_write_u32(smc_dev_t *smc, u32 key, u32 value);

smc_dev_t *smc_init(void);
void smc_shutdown(smc_dev_t *smc);

/*
 * Non-blocking drain of any pending ASC→AP messages on the SMC
 * RTKit endpoint. Safe to call from an FIQ-level tick (no
 * blocking waits, no allocations). Returns the number of messages
 * it drained (0 = idle; >0 = ASC was sending us something). Used
 * by hv.c on T8132 as an experimental M4 keep-alive.
 */
int smc_pump(smc_dev_t *smc);

/*
 * Non-blocking "are you there?" poke: fires a SMC_READ_KEY for
 * the well-known '#KEY' fourcc (which returns the key count and
 * is read-only, safe to issue repeatedly) if there is no previous
 * nudge still outstanding AND the A2I mailbox has room. Otherwise
 * returns without doing anything. Designed for use from an FIQ
 * tick — never blocks, never asserts.
 */
int smc_nudge(smc_dev_t *smc);

/* M4-HV keepalive handle, set by m1n1_main when smc_init succeeds
 * and NULL otherwise. Read by hv.c hv_tick on T8132. */
extern smc_dev_t *hv_smc_keepalive;

#endif
