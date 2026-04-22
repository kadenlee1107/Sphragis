#!/usr/sbin/dtrace -s
/*
 * MTP / AppleASCWrapV6 mailbox trace — live-macOS version.
 *
 * Captures the mailbox/doorbell/power-domain primitives that
 * AppleA7IOP + AppleASCWrapV6 invoke during MTP bring-up. Run on
 * M4 macOS 26.3 as:
 *
 *     sudo dtrace -q -s scripts/macos/trace_mtp_mailbox.d -o mtp.trace
 *
 * Then trigger MTP re-init (sleep/wake usually re-runs AppleA7IOP
 * ::enablePower) to populate the trace, and kill dtrace when done.
 *
 * The output is a time-ordered call log. Feed it to
 * scripts/macos/parse_dtrace_trace.py to emit a Python proxy replay
 * for scripts/hv/boot_mtp_dartmap.py.
 *
 * Probe set:
 *   AppleASCWrapV6::_inbox(void *)              — write 16B to INBOX
 *   AppleASCWrapV6::_outbox(void *)             — read 16B from OUTBOX
 *   AppleASCWrapV6::_triggerFiqNmi()            — doorbell
 *   AppleASCWrapV6::_triggerExtIrqNmi()         — ext doorbell variant
 *   AppleASCWrapV6::_runCPU(bool)               — CPU_CONTROL.RUN
 *   AppleASCWrapV6::_setIORVBAR(uint64_t)       — RVBAR program
 *   AppleASCWrapV6::_mapFirmware(...)           — firmware staging
 *   AppleASCWrapV6::_enableOutbox(bool)         — outbox enable
 *   AppleASCWrapV6::_enableInboxInterrupt(bool)
 *   AppleASCWrapV6::_enableOutboxInterrupt(bool)
 *   AppleASCWrapV6::_disableAllInterrupts()
 *   AppleASCWrapV6::_generateNMI()
 *   AppleASCWrapV6::_getInboxEmpty()            — polled
 *   AppleASCWrapV6::_getInboxFull()
 *   AppleASCWrapV6::_getOutboxEmpty()
 *   AppleASCWrapV6::_isIORVBARLocked()
 *   AppleASCWrapV6::_getKICInboxEnabled()
 *   AppleASCWrapV6::_isIdle(uint32_t *)
 *
 *   AppleA7IOP::start(IOService *)              — driver start (provider)
 *   AppleA7IOP::startCPUWithOptions(fw, opts)   — initial CPU bringup
 *   AppleA7IOP::stopCPU()
 *   AppleA7IOP::_dartMapiBootFirmware(mapper)   — DART init
 *   AppleA7IOP::_dartMapMemoryDescriptor(...)   — DART map
 *   AppleA7IOP::setDoorbellAction(...)          — doorbell handler setup
 *   AppleA7IOP::_inboxHandler(...)              — inbox IRQ handler
 *   AppleA7IOP::_outboxHandler(...)             — outbox IRQ handler
 */

#pragma D option quiet
#pragma D option destructive
#pragma D option bufsize=64m

dtrace:::BEGIN
{
    printf("# MTP/AppleASCWrapV6 trace starting\n");
    printf("# ts_ns event\n");
}

/* ---- AppleASCWrapV6 mailbox primitives ---- */

/*
 * _inbox(this, const void *msg16)
 * Writes 16 bytes at arg1 to the HW inbox mailbox.
 */
fbt::_ZN14AppleASCWrapV66_inboxEPv:entry
{
    printf("[%lld] ASCWrapV6::_inbox this=%p msg=[%016llx %016llx]\n",
           timestamp, (void *)arg0,
           *(uint64_t *)arg1, *((uint64_t *)arg1 + 1));
}

/*
 * _outbox(this, void *dst16)
 * Reads 16 bytes from HW outbox into arg1. Log at return so we see
 * the captured value.
 */
fbt::_ZN14AppleASCWrapV67_outboxEPv:entry
{
    self->outbox_dst = arg1;
    self->outbox_this = arg0;
}

fbt::_ZN14AppleASCWrapV67_outboxEPv:return
/self->outbox_dst != 0/
{
    printf("[%lld] ASCWrapV6::_outbox this=%p msg=[%016llx %016llx]\n",
           timestamp, (void *)self->outbox_this,
           *(uint64_t *)self->outbox_dst,
           *((uint64_t *)self->outbox_dst + 1));
    self->outbox_dst = 0;
    self->outbox_this = 0;
}

/*
 * Doorbell variants.
 */
fbt::_ZN14AppleASCWrapV614_triggerFiqNmiEv:entry
{
    printf("[%lld] ASCWrapV6::_triggerFiqNmi this=%p\n",
           timestamp, (void *)arg0);
}

fbt::_ZN14AppleASCWrapV617_triggerExtIrqNmiEv:entry
{
    printf("[%lld] ASCWrapV6::_triggerExtIrqNmi this=%p\n",
           timestamp, (void *)arg0);
}

fbt::_ZN14AppleASCWrapV612_generateNMIEv:entry
{
    printf("[%lld] ASCWrapV6::_generateNMI this=%p\n",
           timestamp, (void *)arg0);
}

/*
 * CPU control (RUN bit, RVBAR, firmware mapping).
 */
fbt::_ZN14AppleASCWrapV67_runCPUEb:entry
{
    printf("[%lld] ASCWrapV6::_runCPU this=%p enable=%d\n",
           timestamp, (void *)arg0, (int)arg1);
}

fbt::_ZN14AppleASCWrapV611_setIORVBAREy:entry
{
    printf("[%lld] ASCWrapV6::_setIORVBAR this=%p rvbar=0x%llx\n",
           timestamp, (void *)arg0, arg1);
}

fbt::_ZN14AppleASCWrapV612_mapFirmwareEyP18IOMemoryDescriptorj:entry
{
    printf("[%lld] ASCWrapV6::_mapFirmware this=%p addr=0x%llx md=%p flags=0x%x\n",
           timestamp, (void *)arg0, arg1, (void *)arg2, (unsigned int)arg3);
}

fbt::_ZN14AppleASCWrapV614_unmapFirmwareEv:entry
{
    printf("[%lld] ASCWrapV6::_unmapFirmware this=%p\n",
           timestamp, (void *)arg0);
}

/*
 * Interrupt plumbing.
 */
fbt::_ZN14AppleASCWrapV613_enableOutboxEb:entry
{
    printf("[%lld] ASCWrapV6::_enableOutbox this=%p enable=%d\n",
           timestamp, (void *)arg0, (int)arg1);
}

fbt::_ZN14AppleASCWrapV621_enableInboxInterruptEb:entry
{
    printf("[%lld] ASCWrapV6::_enableInboxInterrupt this=%p enable=%d\n",
           timestamp, (void *)arg0, (int)arg1);
}

fbt::_ZN14AppleASCWrapV622_enableOutboxInterruptEb:entry
{
    printf("[%lld] ASCWrapV6::_enableOutboxInterrupt this=%p enable=%d\n",
           timestamp, (void *)arg0, (int)arg1);
}

fbt::_ZN14AppleASCWrapV621_disableAllInterruptsEv:entry
{
    printf("[%lld] ASCWrapV6::_disableAllInterrupts this=%p\n",
           timestamp, (void *)arg0);
}

/*
 * Status polls (noisy but tell us the polling pattern).
 */
fbt::_ZN14AppleASCWrapV614_getInboxEmptyEv:return
{
    printf("[%lld] ASCWrapV6::_getInboxEmpty -> %d\n",
           timestamp, (int)arg1);
}

fbt::_ZN14AppleASCWrapV613_getInboxFullEv:return
{
    printf("[%lld] ASCWrapV6::_getInboxFull -> %d\n",
           timestamp, (int)arg1);
}

fbt::_ZN14AppleASCWrapV615_getOutboxEmptyEv:return
{
    printf("[%lld] ASCWrapV6::_getOutboxEmpty -> %d\n",
           timestamp, (int)arg1);
}

fbt::_ZN14AppleASCWrapV616_isIORVBARLockedEv:return
{
    printf("[%lld] ASCWrapV6::_isIORVBARLocked -> %d\n",
           timestamp, (int)arg1);
}

fbt::_ZN14AppleASCWrapV619_getKICInboxEnabledEv:return
{
    printf("[%lld] ASCWrapV6::_getKICInboxEnabled -> %d\n",
           timestamp, (int)arg1);
}

fbt::_ZN14AppleASCWrapV67_isIdleEPj:return
{
    printf("[%lld] ASCWrapV6::_isIdle -> %d\n",
           timestamp, (int)arg1);
}

/* ---- AppleA7IOP high-level flow ---- */

fbt::_ZN10AppleA7IOP5startEP9IOService:entry
{
    printf("[%lld] AppleA7IOP::start this=%p provider=%p\n",
           timestamp, (void *)arg0, (void *)arg1);
}

fbt::_ZN10AppleA7IOP5startEP9IOService:return
{
    printf("[%lld] AppleA7IOP::start -> %d\n", timestamp, (int)arg1);
}

fbt::_ZN10AppleA7IOP19startCPUWithOptionsEP15IOSlaveFirmwarej:entry
{
    printf("[%lld] AppleA7IOP::startCPUWithOptions this=%p fw=%p opts=0x%x\n",
           timestamp, (void *)arg0, (void *)arg1, (unsigned int)arg2);
}

fbt::_ZN10AppleA7IOP7stopCPUEv:entry
{
    printf("[%lld] AppleA7IOP::stopCPU this=%p\n",
           timestamp, (void *)arg0);
}

fbt::_ZN10AppleA7IOP21_dartMapiBootFirmwareEP8IOMapper:entry
{
    printf("[%lld] AppleA7IOP::_dartMapiBootFirmware this=%p mapper=%p\n",
           timestamp, (void *)arg0, (void *)arg1);
}

fbt::_ZN10AppleA7IOP24_dartMapMemoryDescriptorEP8IOMapperP18IOMemoryDescriptorPy:entry
{
    printf("[%lld] AppleA7IOP::_dartMapMemoryDescriptor this=%p mapper=%p md=%p\n",
           timestamp, (void *)arg0, (void *)arg1, (void *)arg2);
}

fbt::_ZN10AppleA7IOP17setDoorbellActionEPFvP8OSObjectPvjES1_S2_j:entry
{
    printf("[%lld] AppleA7IOP::setDoorbellAction this=%p\n",
           timestamp, (void *)arg0);
}

fbt::_ZN10AppleA7IOP13_inboxHandlerEP22IOInterruptEventSource:entry
{
    printf("[%lld] AppleA7IOP::_inboxHandler this=%p\n",
           timestamp, (void *)arg0);
}

fbt::_ZN10AppleA7IOP14_outboxHandlerEP22IOInterruptEventSource:entry
{
    printf("[%lld] AppleA7IOP::_outboxHandler this=%p\n",
           timestamp, (void *)arg0);
}

fbt::_ZN10AppleA7IOP4_regEj:entry
{
    printf("[%lld] AppleA7IOP::_reg this=%p off=0x%x\n",
           timestamp, (void *)arg0, (unsigned int)arg1);
}

dtrace:::END
{
    printf("# trace ended\n");
}
