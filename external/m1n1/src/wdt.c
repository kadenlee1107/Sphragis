/* SPDX-License-Identifier: MIT */

#include "wdt.h"
#include "adt.h"
#include "soc.h"
#include "types.h"
#include "utils.h"

#define WDT_COUNT 0x10
#define WDT_ALARM 0x14
#define WDT_CTL   0x1c

static u64 wdt_base = 0;

void wdt_kick(void)
{
    if (!wdt_base)
        return;
    write32(wdt_base + WDT_COUNT, 0);
}

void wdt_disable(void)
{
    int path[8];
    int node = adt_path_offset_trace(adt, "/arm-io/wdt", path);

    if (node < 0) {
        printf("WDT node not found!\n");
        return;
    }

    if (adt_get_reg(adt, path, "reg", 0, &wdt_base, NULL)) {
        printf("Failed to get WDT reg property!\n");
        return;
    }

    printf("WDT registers @ 0x%lx\n", wdt_base);

    write32(wdt_base + WDT_CTL, 0);

    /* M4-specific AP watchdog disable.
     *
     * On t8132 the wdt.c standard disable (CTL=0 at reg[0]) isn't enough —
     * the AP WDT still fires at ~118s and reboots the Mac during proxy-only
     * sessions. These four regs come from /arm-io/wdt reg[1..4] on t8132:
     *   reg[1] 0x3882BC224 — AP watchdog deadline (bit 0 = arm)
     *   reg[2] 0x3882B8008 — panicsave
     *   reg[3] 0x3882B802C — panic scratch
     *   reg[4] 0x3882B8020 — (unidentified)
     *
     * Writing 0 to the deadline-arm bit + all-ones to the others placates
     * the WDT indefinitely. Previously this was only done in hv_init(),
     * which doesn't run on proxy-only sessions — so Mac kept rebooting at
     * 118s. Moving it here ensures every m1n1 boot (HV or proxy) stays
     * alive. See docs/SESSION_JOURNAL.md for incident history. */
    if (chip_id == T8132) {
        u32 r1_pre = read32(0x3882BC224UL);
        u32 r2_pre = read32(0x3882B8008UL);
        u32 r3_pre = read32(0x3882B802CUL);
        u32 r4_pre = read32(0x3882B8020UL);
        write32(0x3882BC224UL, 0);
        write32(0x3882B8008UL, 0xffffffff);
        write32(0x3882B802CUL, 0xffffffff);
        write32(0x3882B8020UL, 0xffffffff);
        printf("AP-WDT (t8132): r1 %08x->%08x  r2 %08x->%08x  "
               "r3 %08x->%08x  r4 %08x->%08x\n",
               r1_pre, read32(0x3882BC224UL),
               r2_pre, read32(0x3882B8008UL),
               r3_pre, read32(0x3882B802CUL),
               r4_pre, read32(0x3882B8020UL));
    }

    printf("WDT disabled\n");
}

void wdt_reboot(void)
{
    if (!wdt_base)
        return;

    write32(wdt_base + WDT_ALARM, 0x100000);
    write32(wdt_base + WDT_COUNT, 0);
    write32(wdt_base + WDT_CTL, 4);
}
