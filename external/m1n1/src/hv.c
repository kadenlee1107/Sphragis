/* SPDX-License-Identifier: MIT */

#include "hv.h"
#include "assert.h"
#include "cpu_regs.h"
#include "display.h"
#include "gxf.h"
#include "memory.h"
#include "pcie.h"
#include "smp.h"
#include "soc.h"
#include "smc.h"
#include "string.h"
#include "usb.h"
#include "utils.h"
#include "wdt.h"

#define HV_TICK_RATE      1000
#define HV_SLOW_TICK_RATE 1

DECLARE_SPINLOCK(bhl);

void hv_enter_guest(u64 x0, u64 x1, u64 x2, u64 x3, void *entry);
void hv_exit_guest(void) __attribute__((noreturn));

extern char _hv_vectors_start[0];

u64 hv_tick_interval;
u64 hv_secondary_tick_interval;

int hv_pinned_cpu;
int hv_want_cpu;

static bool hv_has_ecv;
static bool hv_should_exit[MAX_CPUS];
bool hv_started_cpus[MAX_CPUS];
u64 hv_cpus_in_guest;
u64 hv_saved_sp[MAX_CPUS];

struct hv_secondary_info_t {
    uint64_t hcr;
    uint64_t hacr;
    uint64_t vtcr, vttbr;
    uint64_t mdcr;
    uint64_t mdscr;
    uint64_t amx_ctl;
    uint64_t apvmkeylo, apvmkeyhi, apsts;
    uint64_t actlr_el2;
    uint64_t actlr_el1;
    uint64_t cnthctl;
    uint64_t sprr_config;
    uint64_t gxf_config;
};

static struct hv_secondary_info_t hv_secondary_info;

void hv_init(void)
{
    // Bat_OS M4 instrumentation: markers between every hv_init step
    // so a chainload + run_guest.py that hangs can tell us exactly
    // which line trapped. Next session greps the log for the last
    // marker and gates that Apple-private MSR behind a chip_id check.
    printf("[hv_init] M0 enter\n");
#ifndef BATOS_HV_BISECT_NO_QUIESCE
    pcie_shutdown();
    printf("[hv_init] M1 pcie_shutdown\n");
    // Make sure we wake up DCP if we put it to sleep, just quiesce it to match ADT
    if (display_is_external && display_start_dcp() >= 0)
        display_shutdown(DCP_QUIESCED);
    printf("[hv_init] M2 display_shutdown\n");
    // reenable hpm interrupts for the guest for unused iodevs
    usb_hpm_restore_irqs(0);
    printf("[hv_init] M3 usb_hpm_restore\n");
#else
    printf("[hv_init] M1-M3 SKIPPED (BATOS_HV_BISECT_NO_QUIESCE) "
           "— pcie/display/usb quiesce bypassed\n");
#endif
    smp_start_secondaries();
    printf("[hv_init] M4 smp_start_secondaries\n");
    smp_set_wfe_mode(true);
    printf("[hv_init] M5 smp_set_wfe_mode\n");
    hv_wdt_init();
    printf("[hv_init] M6 hv_wdt_init\n");

    hv_exc_stats_init();
    printf("[hv_init] M6.5 hv_exc_stats_init\n");

    hv_pt_init();
    printf("[hv_init] M7 hv_pt_init\n");

    // Configure hypervisor defaults
    hv_write_hcr(HCR_API | // Allow PAuth instructions
                 HCR_APK | // Allow PAuth key registers
                 HCR_TEA | // Trap external aborts
                 HCR_E2H | // VHE mode (forced)
                 HCR_RW |  // AArch64 guest
                 HCR_AMO | // Trap SError exceptions
                 HCR_VM);  // Enable stage 2 translation
    printf("[hv_init] M8 hv_write_hcr\n");

    // No guest vectors initially
    msr(VBAR_EL12, 0);
    printf("[hv_init] M9 msr VBAR_EL12\n");

    // Compute tick interval
    hv_tick_interval = mrs(CNTFRQ_EL0) / HV_TICK_RATE;
    printf("[hv_init] M10 tick_interval\n");

    hv_has_ecv = mrs(ID_AA64MMFR0_EL1) & (0xfULL << 60);
    printf("[hv_init] M11 has_ecv=%d\n", (int)hv_has_ecv);

    if (hv_has_ecv) {
        printf("HV: ECV enabled\n");
        reg_set(CNTHCTL_EL2,
                CNTHCTL_EL1NVVCT | CNTHCTL_EL1NVPCT | CNTHCTL_EL1TVT | CNTHCTL_EL1PCTEN);
        hv_secondary_tick_interval = mrs(CNTFRQ_EL0) / HV_SLOW_TICK_RATE;
    } else {
        printf("HV: No ECV supported\n");
        // Enable physical timer for EL1
        msr(CNTHCTL_EL2, CNTHCTL_EL1PTEN | CNTHCTL_EL1PCTEN);

        hv_secondary_tick_interval = hv_tick_interval;
    }
    printf("[hv_init] M12 CNTHCTL set\n");

    // Set deep WFI back to defaults
    if (cpu_features->cyc_ovrd)
        reg_mask(SYS_IMP_APL_CYC_OVRD, CYC_OVRD_WFI_MODE_MASK, CYC_OVRD_WFI_MODE(0));
    printf("[hv_init] M13 cyc_ovrd\n");

    sysop("dsb ishst");
    sysop("tlbi alle1is");
    sysop("dsb ish");
    sysop("isb");
    printf("[hv_init] M14 done\n");
}

static void hv_set_gxf_vbar(void)
{
    msr(SYS_IMP_APL_VBAR_GL1, _hv_vectors_start);
}

void hv_start(void *entry, u64 regs[4])
{
    if (boot_cpu_idx == -1) {
        printf("Boot CPU has not been found, can't start hypervisor\n");
        return;
    }

    printf("[hv_start] S0 enter\n");
    memset(hv_should_exit, 0, sizeof(hv_should_exit));
    memset(hv_started_cpus, 0, sizeof(hv_started_cpus));

    hv_started_cpus[boot_cpu_idx] = true;

    msr(VBAR_EL1, _hv_vectors_start);
    printf("[hv_start] S1 VBAR_EL1\n");

    if (gxf_enabled())
        gl2_call(hv_set_gxf_vbar, 0, 0, 0, 0);
    printf("[hv_start] S2 gxf_vbar\n");

    hv_secondary_info.hcr = mrs(HCR_EL2);
    hv_secondary_info.hacr = mrs(HACR_EL2);
    hv_secondary_info.vtcr = mrs(VTCR_EL2);
    hv_secondary_info.vttbr = mrs(VTTBR_EL2);
    hv_secondary_info.mdcr = mrs(MDCR_EL2);
    hv_secondary_info.mdscr = mrs(MDSCR_EL1);
    printf("[hv_start] S3 arch MRS\n");
    if (cpu_features->amx) {
        hv_secondary_info.amx_ctl = mrs(SYS_IMP_APL_AMX_CTL_EL2);
        hv_secondary_info.apvmkeylo = mrs(SYS_IMP_APL_APVMKEYLO_EL2);
        hv_secondary_info.apvmkeyhi = mrs(SYS_IMP_APL_APVMKEYHI_EL2);
        hv_secondary_info.apsts = mrs(SYS_IMP_APL_APSTS_EL12);
    } else {
        hv_secondary_info.amx_ctl = 0;
        hv_secondary_info.apvmkeylo = 0;
        hv_secondary_info.apvmkeyhi = 0;
        hv_secondary_info.apsts = 0;
    }
    printf("[hv_start] S4 AMX/VMKEY MRS\n");
    hv_secondary_info.actlr_el2 = mrs(ACTLR_EL2);
    if (cpu_features->actlr_el2)
        hv_secondary_info.actlr_el1 = mrs(SYS_ACTLR_EL12);
    else
        hv_secondary_info.actlr_el1 = mrs(SYS_IMP_APL_ACTLR_EL12);
    hv_secondary_info.cnthctl = mrs(CNTHCTL_EL2);
    printf("[hv_start] S5 ACTLR/CNTHCTL MRS\n");
    if (cpu_features->mmu_sprr) {
        hv_secondary_info.sprr_config = mrs(SYS_IMP_APL_SPRR_CONFIG_EL1);
        hv_secondary_info.gxf_config = mrs(SYS_IMP_APL_GXF_CONFIG_EL1);
    } else {
        hv_secondary_info.sprr_config = 0;
        hv_secondary_info.gxf_config = 0;
    }
    printf("[hv_start] S6 SPRR/GXF MRS\n");

    // M4-HV (2026-04-20 re-try): the SErrors from PL011 / rodata
    // absolutes / vuart-FB-deadlock that used to mask any benefit
    // of the tick are now fixed. Hypothesis: with those gone, a
    // 1 kHz hv_tick() drain of IODEV_USB_VUART may keep whatever
    // wall-clock watchdog (suspected SMC/AOP) alive long enough
    // for multi-minute sessions. Re-enable on T8132 and measure.
    // M4-HV (2026-04-20): re-enabled on T8132. Back-to-back A/B on
    // an otherwise-identical build showed tick-enabled extended the
    // USB drop from ~60 s (control) to ~86 s under BATOS_KEEP_FB=1
    // + polled stimulus. The earlier "tick doesn't help" result (see
    // 2026-04-19 22:30 journal) predated the PL011/rodata/vuart-FB
    // fixes that were masking the benefit. Guest path is clean: no
    // SError, heartbeats monotonic, uptime reports correctly. Session
    // length is still wall-clock capped (SMC/AOP heartbeat suspected)
    // but tick buys ~40% more budget basically for free.
    hv_arm_tick(false);
    printf("[hv_start] S7 hv_arm_tick armed (m4 re-enabled)\n");
    hv_pinned_cpu = -1;
    hv_want_cpu = -1;
    hv_cpus_in_guest = BIT(smp_id());
    printf("[hv_start] S8 entering guest @ %p x0=%lx\n", entry, regs[0]);
    // Flush the serial console before eret so any buffered markers
    // actually reach the host (no CNTP tick drives flush on M4 now).
    iodev_console_flush();

    hv_enter_guest(regs[0], regs[1], regs[2], regs[3], entry);

    __atomic_and_fetch(&hv_cpus_in_guest, ~BIT(smp_id()), __ATOMIC_ACQUIRE);
    spin_lock(&bhl);

    hv_wdt_stop();

    printf("HV: Exiting hypervisor (main CPU)\n");

    spin_unlock(&bhl);
    // Wait a bit for the guest CPUs to exit on their own if they are in the process.
    udelay(200000);
    spin_lock(&bhl);

    hv_started_cpus[boot_cpu_idx] = false;

    for (int i = 0; i < MAX_CPUS; i++) {
        if (i == boot_cpu_idx) {
            continue;
        }
        hv_should_exit[i] = true;
        if (hv_started_cpus[i]) {
            printf("HV: Waiting for CPU %d to exit\n", i);
            spin_unlock(&bhl);
            smp_wait(i);
            spin_lock(&bhl);
            hv_started_cpus[i] = false;
        }
    }

    printf("HV: All CPUs exited\n");
    spin_unlock(&bhl);
}

static void hv_init_secondary(struct hv_secondary_info_t *info)
{
    gxf_init();

    msr(VBAR_EL1, _hv_vectors_start);

    msr(HCR_EL2, info->hcr);
    msr(HACR_EL2, info->hacr);
    msr(VTCR_EL2, info->vtcr);
    msr(VTTBR_EL2, info->vttbr);
    msr(MDCR_EL2, info->mdcr);
    msr(MDSCR_EL1, info->mdscr);
    if (cpu_features->amx) {
        msr(SYS_IMP_APL_AMX_CTL_EL2, info->amx_ctl);
        msr(SYS_IMP_APL_APVMKEYLO_EL2, info->apvmkeylo);
        msr(SYS_IMP_APL_APVMKEYHI_EL2, info->apvmkeyhi);
        msr(SYS_IMP_APL_APSTS_EL12, info->apsts);
    }
    msr(ACTLR_EL2, info->actlr_el2);
    if (cpu_features->actlr_el2)
        msr(SYS_ACTLR_EL12, info->actlr_el1);
    else
        msr(SYS_IMP_APL_ACTLR_EL12, info->actlr_el1);
    msr(CNTHCTL_EL2, info->cnthctl);
    if (cpu_features->mmu_sprr) {
        msr(SYS_IMP_APL_SPRR_CONFIG_EL1, info->sprr_config);
        msr(SYS_IMP_APL_GXF_CONFIG_EL1, info->gxf_config);
    }

    if (cpu_features->cyc_ovrd)
        reg_mask(SYS_IMP_APL_CYC_OVRD, CYC_OVRD_WFI_MODE_MASK, CYC_OVRD_WFI_MODE(0));

    if (gxf_enabled())
        gl2_call(hv_set_gxf_vbar, 0, 0, 0, 0);

    hv_arm_tick(true);
}

static void hv_enter_secondary(void *entry, u64 regs[4])
{
    hv_enter_guest(regs[0], regs[1], regs[2], regs[3], entry);

    spin_lock(&bhl);

    printf("HV: Exiting from CPU %d\n", smp_id());

    __atomic_and_fetch(&hv_cpus_in_guest, ~BIT(smp_id()), __ATOMIC_ACQUIRE);

    hv_started_cpus[smp_id()] = false;
    spin_unlock(&bhl);
}

void hv_start_secondary(int cpu, void *entry, u64 regs[4])
{
    printf("HV: Initializing secondary %d\n", cpu);
    iodev_console_flush();

    mmu_init_secondary(cpu);
    iodev_console_flush();
    smp_call4(cpu, hv_init_secondary, (u64)&hv_secondary_info, 0, 0, 0);
    smp_wait(cpu);
    iodev_console_flush();

    printf("HV: Entering guest secondary %d at %p\n", cpu, entry);
    hv_started_cpus[cpu] = true;
    __atomic_or_fetch(&hv_cpus_in_guest, BIT(smp_id()), __ATOMIC_ACQUIRE);

    iodev_console_flush();
    smp_call4(cpu, hv_enter_secondary, (u64)entry, (u64)regs, 0, 0);
}

void hv_exit_cpu(int cpu)
{
    if (cpu == -1)
        cpu = smp_id();

    printf("HV: Requesting exit of CPU#%d from the guest\n", cpu);
    hv_should_exit[cpu] = true;
}

void hv_rendezvous(void)
{
    int timeout = 1000000;

    if (!__atomic_load_n(&hv_cpus_in_guest, __ATOMIC_ACQUIRE))
        return;

    /* IPI all CPUs. This might result in spurious IPIs to the guest... */
    for (int i = 0; i < MAX_CPUS; i++) {
        if (i != smp_id() && hv_started_cpus[i]) {
            smp_send_ipi(i);
        }
    }

    while (timeout--) {
        if (!__atomic_load_n(&hv_cpus_in_guest, __ATOMIC_ACQUIRE))
            return;
    }

    hv_panic("HV: Failed to rendezvous, missing CPUs: 0x%lx (current: %d)\n",
             __atomic_load_n(&hv_cpus_in_guest, __ATOMIC_ACQUIRE), smp_id());
}

bool hv_switch_cpu(int cpu)
{
    if (cpu > MAX_CPUS || cpu < 0 || !hv_started_cpus[cpu]) {
        printf("HV: CPU #%d is inactive or invalid\n", cpu);
        return false;
    }
    printf("HV: switching to CPU #%d\n", cpu);
    hv_want_cpu = cpu;
    hv_rendezvous();
    return true;
}

void hv_pin_cpu(int cpu)
{
    hv_pinned_cpu = cpu;
}

void hv_write_hcr(u64 val)
{
    if (gxf_enabled() && !in_gl12())
        gl2_call(hv_write_hcr, val, 0, 0, 0);
    else
        msr(HCR_EL2, val);
}

u64 hv_get_spsr(void)
{
    if (in_gl12())
        return mrs(SYS_IMP_APL_SPSR_GL1);
    else
        return mrs(SPSR_EL2);
}

void hv_set_spsr(u64 val)
{
    if (in_gl12())
        return msr(SYS_IMP_APL_SPSR_GL1, val);
    else
        return msr(SPSR_EL2, val);
}

u64 hv_get_esr(void)
{
    if (in_gl12())
        return mrs(SYS_IMP_APL_ESR_GL1);
    else
        return mrs(ESR_EL2);
}

u64 hv_get_far(void)
{
    if (in_gl12())
        return mrs(SYS_IMP_APL_FAR_GL1);
    else
        return mrs(FAR_EL2);
}

u64 hv_get_afsr1(void)
{
    if (in_gl12())
        return mrs(SYS_IMP_APL_AFSR1_GL1);
    else
        return mrs(AFSR1_EL2);
}

u64 hv_get_elr(void)
{
    if (in_gl12())
        return mrs(SYS_IMP_APL_ELR_GL1);
    else
        return mrs(ELR_EL2);
}

void hv_set_elr(u64 val)
{
    if (in_gl12())
        return msr(SYS_IMP_APL_ELR_GL1, val);
    else
        return msr(ELR_EL2, val);
}

void hv_arm_tick(bool secondary)
{
    if (secondary)
        msr(CNTP_TVAL_EL0, hv_secondary_tick_interval);
    else
        msr(CNTP_TVAL_EL0, hv_tick_interval);
    msr(CNTP_CTL_EL0, CNTx_CTL_ENABLE);
}

void hv_maybe_exit(void)
{
    if (hv_should_exit[smp_id()]) {
        hv_exit_guest();
    }
}

void hv_tick(struct exc_info *ctx)
{
    hv_wdt_pet();
    /* M4-HV diagnostic: per-CPU event stats every ~2 s. Only the
     * interruptible (boot) CPU reaches hv_tick in the steady state,
     * which is also the only CPU that has smp-consistent access to
     * every other CPU's pcpu slot. */
    hv_exc_stats_snapshot();
    iodev_handle_events(uartproxy_iodev);
    if (iodev_can_read(uartproxy_iodev)) {
        printf("HV: User interrupt\n");
        iodev_console_flush();
        if (hv_pinned_cpu == -1 || hv_pinned_cpu == smp_id())
            hv_exc_proxy(ctx, START_HV, HV_USER_INTERRUPT, NULL);
    }
    // M4-HV: hv_vuart_poll() calls aic_set_sw() on AIC v3 to inject
    // a software IRQ to the guest. On M4 that write pokes AIC state
    // that was already configured by m1n1 and triggers a slow reset
    // (~30-60 s). Our guest has no AIC configured under HV anyway
    // (bring_up_all is gated to skip AIC on M4), so skip the IRQ
    // injection — the dockchannel-vuart MMIO trap path (hv_vuart.c
    // handle_vuart_dockchannel) delivers TX/RX bytes without needing
    // an IRQ line into the guest.
    // Keep the poll on non-M4 SoCs so Linux / XNU guests still get
    // UART IRQs as expected.
    if (chip_id != T8132)
        hv_vuart_poll();
    else {
        iodev_handle_events(IODEV_USB_VUART);
        // M4-HV: kick the SoC WDT countdown register (defensive; see
        // 2026-04-20 11:35 journal — the ADT-declared WDT isn't the
        // reset trigger but the write is free).
        wdt_kick();
        // M4-HV 2026-04-20: SMC from HV context wedges guest even
        // with AIC drain. Not called here. hv_exc_fiq still drains
        // AIC events on T8132 defensively.
    }
}
