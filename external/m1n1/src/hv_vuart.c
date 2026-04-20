/* SPDX-License-Identifier: MIT */

#include "hv.h"
#include "aic.h"
#include "iodev.h"
#include "uart.h"
#include "uart_regs.h"
#include "uartproxy.h"
#include "usb.h"

bool active = false;

u32 ucon = 0;
u32 utrstat = 0;
u32 ufstat = 0;

int vuart_irq = 0;

static void update_irq(void)
{
    ssize_t rx_queued;

    iodev_handle_events(IODEV_USB_VUART);

    utrstat |= UTRSTAT_TXBE | UTRSTAT_TXE;
    utrstat &= ~UTRSTAT_RXD;

    ufstat = 0;
    if ((rx_queued = iodev_can_read(IODEV_USB_VUART))) {
        utrstat |= UTRSTAT_RXD;
        if (rx_queued > 15)
            ufstat = FIELD_PREP(UFSTAT_RXCNT, 15) | UFSTAT_RXFULL;
        else
            ufstat = FIELD_PREP(UFSTAT_RXCNT, rx_queued);

        if (FIELD_GET(UCON_RXMODE, ucon) == UCON_MODE_IRQ && ucon & UCON_RXTO_ENA) {
            utrstat |= UTRSTAT_RXTO;
        }
    }

    if (FIELD_GET(UCON_TXMODE, ucon) == UCON_MODE_IRQ && ucon & UCON_TXTHRESH_ENA) {
        utrstat |= UTRSTAT_TXTHRESH;
    }

    if (vuart_irq) {
        uart_clear_irqs();
        if (utrstat & (UTRSTAT_TXTHRESH | UTRSTAT_RXTHRESH | UTRSTAT_RXTO)) {
            aic_set_sw(vuart_irq, true);
        } else {
            aic_set_sw(vuart_irq, false);
        }
    }

    //     printf("HV: vuart UTRSTAT=0x%x UFSTAT=0x%x UCON=0x%x\n", utrstat, ufstat, ucon);
}

static void handle_vuart_passthrough(uint8_t b)
{
    const char PREFIX[] = "HVLOG: ";
    static int state = 0;

    if (!PREFIX[state]) {
        if (b == '\r' || b == '\n') {
            printf("\n");
            state = 0;
            return;
        }
        printf("%c", b);
        return;
    }

    if (b == PREFIX[state])
        state++;
    else
        state = 0;

    if (!PREFIX[state])
        printf("%s", PREFIX);
}

static bool handle_vuart(struct exc_info *ctx, u64 addr, u64 *val, bool write, int width)
{
    UNUSED(ctx);
    UNUSED(width);

    addr &= 0xfff;

    update_irq();

    if (write) {
        //         printf("HV: vuart W 0x%lx <- 0x%lx (%d)\n", addr, *val, width);
        switch (addr) {
            case UCON:
                ucon = *val;
                break;
            case UTXH: {
                uint8_t b = *val;
                if (iodev_can_write(IODEV_USB_VUART))
                    iodev_write(IODEV_USB_VUART, &b, 1);
                handle_vuart_passthrough(b);
                break;
            }
            case UTRSTAT:
                utrstat &= ~(*val & (UTRSTAT_TXTHRESH | UTRSTAT_RXTHRESH | UTRSTAT_RXTO));
                break;
        }
    } else {
        switch (addr) {
            case UCON:
                *val = ucon;
                break;
            case URXH:
                if (iodev_can_read(IODEV_USB_VUART)) {
                    uint8_t c;
                    iodev_read(IODEV_USB_VUART, &c, 1);
                    *val = c;
                } else {
                    *val = 0;
                }
                break;
            case UTRSTAT:
                *val = utrstat;
                break;
            case UFSTAT:
                *val = ufstat;
                break;
            default:
                *val = 0;
                break;
        }
        //         printf("HV: vuart R 0x%lx -> 0x%lx (%d)\n", addr, *val, width);
    }

    return true;
}

void hv_vuart_poll(void)
{
    if (!active)
        return;

    update_irq();
}

void hv_map_vuart(u64 base, int irq, iodev_id_t iodev)
{
    hv_map_hook(base, handle_vuart, 0x1000);
    usb_iodev_vuart_setup(iodev);
    vuart_irq = irq;
    active = true;
}

// M4 dockchannel UART register layout (mirror of
// external/m1n1/src/dockchannel_uart.c). Trap guest reads/writes in
// the dockchannel region and forward byte traffic to the same
// IODEV_USB_VUART that the Samsung-style uart0 vuart uses, so a
// Bat_OS guest writing to 0x3_8812_8000 ends up on /dev/ttyACM2.
#define DC_DATA_TX8       0x4004
#define DC_DATA_TX_FREE   0x4014
#define DC_DATA_RX8       0x401c
#define DC_DATA_RX_COUNT  0x402c

static u64 vuart_dc_base = 0;

static bool handle_vuart_dockchannel(struct exc_info *ctx, u64 addr, u64 *val,
                                     bool write, int width)
{
    UNUSED(ctx);
    UNUSED(width);

    // Heartbeat: print "HV alive t=Ns" once per second so we can
    // distinguish m1n1 crashing vs USB CDC stalling vs external
    // watchdog reset when a long session ends.
    {
        static u64 hv_start_ts = 0;
        static u64 last_heartbeat = 0;
        u64 now = mrs(CNTPCT_EL0);
        if (hv_start_ts == 0)
            hv_start_ts = now;
        u64 freq = mrs(CNTFRQ_EL0);
        if (freq > 0 && (now - last_heartbeat) > freq) {
            u64 elapsed_sec = (now - hv_start_ts) / freq;
            printf("HV alive t=%lus\n", elapsed_sec);
            last_heartbeat = now;
        }
    }

    u64 off = addr - vuart_dc_base;

    if (write) {
        switch (off) {
            case DC_DATA_TX8: {
                uint8_t b = *val;
                if (iodev_can_write(IODEV_USB_VUART))
                    iodev_write(IODEV_USB_VUART, &b, 1);
                handle_vuart_passthrough(b);
                break;
            }
            default:
                // Silently drop writes we don't understand — the
                // dockchannel register space also has clock / state
                // control registers that we don't need to virtualise
                // for a read-only stdout path.
                break;
        }
    } else {
        switch (off) {
            case DC_DATA_TX_FREE:
                // Claim an always-ready TX FIFO. iodev_write will
                // buffer internally or drop on overflow.
                *val = 0x100;
                break;
            case DC_DATA_RX8: {
                iodev_handle_events(IODEV_USB_VUART);
                // Also drain the proxy endpoint's DWC3 events. Without
                // periodic service, the USB CDC stalls after ~30 s
                // (suspected Apple SMC heartbeat watchdog). Guest
                // polls this register in a tight loop via apple_serial_shell,
                // so every has_char() call also pets the USB stack.
                iodev_handle_events(uartproxy_iodev);
                if (iodev_can_read(IODEV_USB_VUART) > 0) {
                    uint8_t c;
                    if (iodev_read(IODEV_USB_VUART, &c, 1) == 1) {
                        *val = ((u64)c) << 8;
                        break;
                    }
                }
                *val = 0;
                break;
            }
            case DC_DATA_RX_COUNT: {
                iodev_handle_events(IODEV_USB_VUART);
                iodev_handle_events(uartproxy_iodev);
                *val = iodev_can_read(IODEV_USB_VUART) > 0 ? 1 : 0;
                break;
            }
            default:
                *val = 0;
                break;
        }
    }

    return true;
}

void hv_map_vuart_dockchannel(u64 base, iodev_id_t iodev)
{
    vuart_dc_base = base;
    // Dockchannel reg block on M4 is larger than 0x8000: the TX/RX
    // FIFO pair sits at +0x4000 within a +0x10000 region. Map enough.
    hv_map_hook(base, handle_vuart_dockchannel, 0x10000);
    usb_iodev_vuart_setup(iodev);
    // Drain any stale bytes from the host→device ring so the guest
    // shell doesn't see leftover data from before the hook was armed.
    iodev_handle_events(IODEV_USB_VUART);
    while (iodev_can_read(IODEV_USB_VUART) > 0) {
        uint8_t dump[64];
        ssize_t n = iodev_read(IODEV_USB_VUART, dump, sizeof(dump));
        if (n <= 0)
            break;
    }
    active = true;
}
