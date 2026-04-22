#!/usr/sbin/dtrace -s
/*
 * MTP / AppleA7IOP mailbox trace — live-macOS version.
 *
 * The ASCWrapV6-specific `_inbox`/`_outbox`/`_triggerFiqNmi`/etc. got
 * inlined at build time and aren't in the live kernel's fbt provider
 * (confirmed on macOS 26.3, M4 J604). Their AppleA7IOP-level wrappers
 * ARE present, so we probe those instead — which gives us nicer
 * semantic info anyway (which mailbox, read vs write, size, etc.).
 *
 * Run as:
 *     sudo dtrace -q -s /tmp/trace_mtp_mailbox.d -o /tmp/mtp_init.trace
 *
 * Trigger MTP activity (sleep/wake, or just type on the keyboard)
 * during the run. Ctrl-C or use the exit-after-N-sec pattern to
 * stop.
 *
 * Output: time-ordered mailbox op log. Feed to
 * scripts/macos/parse_dtrace_trace.py to emit an m1n1-proxy replay.
 */

#pragma D option quiet
#pragma D option destructive
#pragma D option bufsize=64m

dtrace:::BEGIN
{
    printf("# MTP/AppleA7IOP trace starting\n");
}

/*
 * AppleA7IOP::postMailbox(mbox_id, data, size, wait) — write.
 * Dump the first 16 bytes of the message since MTP mailbox
 * messages are 8 or 16 bytes.
 */
fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP11postMailboxEjPvjb:entry
{
    printf("[%llu] A7IOP::postMailbox this=%p mbox=%u size=%u wait=%d\n",
           timestamp, (void *)arg0, (unsigned)arg1, (unsigned)arg3, (int)arg4);
    printf("    data=[%016llx %016llx]\n",
           *(uint64_t *)arg2, *((uint64_t *)arg2 + 1));
}

/*
 * AppleA7IOP::getMailbox(mbox_id, buf, wait) — read. Dump on return
 * so we see the captured value.
 */
fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP10getMailboxEjPvb:entry
{
    self->getm_this = arg0;
    self->getm_mbox = arg1;
    self->getm_buf  = arg2;
}

fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP10getMailboxEjPvb:return
/self->getm_buf != 0/
{
    printf("[%llu] A7IOP::getMailbox this=%p mbox=%u -> [%016llx %016llx]\n",
           timestamp, (void *)self->getm_this, (unsigned)self->getm_mbox,
           *(uint64_t *)self->getm_buf,
           *((uint64_t *)self->getm_buf + 1));
    self->getm_this = 0;
    self->getm_mbox = 0;
    self->getm_buf  = 0;
}

/*
 * AppleA7IOP::getMailboxBulk(buf, &size) — bulk read.
 */
fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP14getMailboxBulkEPvPj:entry
{
    self->gb_this = arg0;
    self->gb_buf  = arg1;
    self->gb_size = arg2;
}

fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP14getMailboxBulkEPvPj:return
/self->gb_buf != 0/
{
    printf("[%llu] A7IOP::getMailboxBulk this=%p size=%u [%016llx %016llx]\n",
           timestamp, (void *)self->gb_this,
           *(uint32_t *)self->gb_size,
           *(uint64_t *)self->gb_buf,
           *((uint64_t *)self->gb_buf + 1));
    self->gb_this = 0;
    self->gb_buf  = 0;
    self->gb_size = 0;
}

/*
 * Doorbell, polling wait, interrupt gating.
 */
fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP12ringDoorbellEj:entry
{
    printf("[%llu] A7IOP::ringDoorbell this=%p mbox=%u\n",
           timestamp, (void *)arg0, (unsigned)arg1);
}

fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP14waitForMailboxEj:entry
{
    printf("[%llu] A7IOP::waitForMailbox this=%p mbox=%u\n",
           timestamp, (void *)arg0, (unsigned)arg1);
}

fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP23enableMailboxInterruptsEb:entry
{
    printf("[%llu] A7IOP::enableMailboxInterrupts this=%p enable=%d\n",
           timestamp, (void *)arg0, (int)arg1);
}

fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP17setDoorbellActionEPFvP8OSObjectPvjES1_S2_j:entry
{
    printf("[%llu] A7IOP::setDoorbellAction this=%p action=%p ctx=%p\n",
           timestamp, (void *)arg0, (void *)arg1, (void *)arg2);
}

/*
 * Start / stop / power.
 */
fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP5startEP9IOService:entry
{
    printf("[%llu] A7IOP::start this=%p provider=%p\n",
           timestamp, (void *)arg0, (void *)arg1);
}

fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP5startEP9IOService:return
{
    printf("[%llu] A7IOP::start -> %d\n", timestamp, (int)arg1);
}

fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP11enablePowerEv:entry
{
    printf("[%llu] A7IOP::enablePower this=%p\n", timestamp, (void *)arg0);
}

fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP11enablePowerEv:return
{
    printf("[%llu] A7IOP::enablePower -> %d\n", timestamp, (int)arg1);
}

fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP13_disablePowerEv:entry
{
    printf("[%llu] A7IOP::_disablePower this=%p\n", timestamp, (void *)arg0);
}

fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP16disablePowerLateEv:entry
{
    printf("[%llu] A7IOP::disablePowerLate this=%p\n", timestamp, (void *)arg0);
}

fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP19startCPUWithOptionsEP15IOSlaveFirmwarej:entry
{
    printf("[%llu] A7IOP::startCPUWithOptions this=%p fw=%p opts=0x%x\n",
           timestamp, (void *)arg0, (void *)arg1, (unsigned)arg2);
}

fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP7stopCPUEb:entry
{
    printf("[%llu] A7IOP::stopCPU this=%p force=%d\n",
           timestamp, (void *)arg0, (int)arg1);
}

fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP7_runCPUEb:entry
{
    printf("[%llu] A7IOP::_runCPU this=%p enable=%d\n",
           timestamp, (void *)arg0, (int)arg1);
}

fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP12_generateNMIEv:entry
{
    printf("[%llu] A7IOP::_generateNMI this=%p\n", timestamp, (void *)arg0);
}

fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP16_syncIOPTimebaseEv:entry
{
    printf("[%llu] A7IOP::_syncIOPTimebase this=%p\n", timestamp, (void *)arg0);
}

/*
 * Firmware mapping.
 */
fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP12_mapFirmwareEyP18IOMemoryDescriptorj:entry
{
    printf("[%llu] A7IOP::_mapFirmware this=%p addr=0x%llx md=%p flags=0x%x\n",
           timestamp, (void *)arg0, arg1, (void *)arg2, (unsigned)arg3);
}

fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP14_unmapFirmwareEv:entry
{
    printf("[%llu] A7IOP::_unmapFirmware this=%p\n", timestamp, (void *)arg0);
}

fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP21_dartMapiBootFirmwareEP8IOMapper:entry
{
    printf("[%llu] A7IOP::_dartMapiBootFirmware this=%p mapper=%p\n",
           timestamp, (void *)arg0, (void *)arg1);
}

fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP24_dartMapMemoryDescriptorEP8IOMapperP18IOMemoryDescriptorPy:entry
{
    printf("[%llu] A7IOP::_dartMapMemoryDescriptor this=%p mapper=%p md=%p\n",
           timestamp, (void *)arg0, (void *)arg1, (void *)arg2);
}

/*
 * IRQ handlers.
 */
fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP13_inboxHandlerEP22IOInterruptEventSource:entry
{
    printf("[%llu] A7IOP::_inboxHandler this=%p\n", timestamp, (void *)arg0);
}

fbt:com.apple.driver.AppleA7IOP:_ZN10AppleA7IOP14_outboxHandlerEP22IOInterruptEventSource:entry
{
    printf("[%llu] A7IOP::_outboxHandler this=%p\n", timestamp, (void *)arg0);
}

/*
 * ASCWrapV6-level probes that DID survive inlining. These give us
 * the MTP-specific (vs SMC-specific) override points.
 */
fbt:com.apple.driver.AppleA7IOP-ASCWrap-v6:_ZN14AppleASCWrapV6*:entry
{
    printf("[%llu] ASCWrapV6 enter func=%s this=%p arg1=%llx\n",
           timestamp, probefunc, (void *)arg0, arg1);
}

dtrace:::END
{
    printf("# trace ended\n");
}
