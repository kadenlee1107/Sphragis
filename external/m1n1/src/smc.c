/* SPDX-License-Identifier: MIT */

#include "asc.h"
#include "assert.h"
#include "malloc.h"
#include "smc.h"
#include "string.h"
#include "types.h"
#include "utils.h"

#define SMC_READ_KEY         0x10
#define SMC_WRITE_KEY        0x11
#define SMC_GET_KEY_BY_INDEX 0x12
#define SMC_GET_KEY_INFO     0x13
#define SMC_INITIALIZE       0x17
#define SMC_NOTIFICATION     0x18
#define SMC_RW_KEY           0x20

#define SMC_MSG_TYPE GENMASK(7, 0)
#define SMC_MSG_ID   GENMASK(15, 12)

#define SMC_WRITE_KEY_SIZE GENMASK(23, 16)
#define SMC_WRITE_KEY_KEY  GENMASK(63, 32)

#define SMC_RESULT_RESULT GENMASK(7, 0)
#define SMC_RESULT_ID     GENMASK(15, 12)
#define SMC_RESULT_SIZE   GENMASK(31, 16)
#define SMC_RESULT_VALUE  GENMASK(63, 32)

#define SMC_NUM_IDS 16

#define SMC_ENDPOINT 0x20

struct smc_dev {
    asc_dev_t *asc;
    rtkit_dev_t *rtkit;

    void *shmem;
    u32 msgid;

    bool outstanding[SMC_NUM_IDS];
    u64 ret[SMC_NUM_IDS];
};

smc_dev_t *hv_smc_keepalive = NULL;

static void smc_handle_msg(smc_dev_t *smc, u64 msg)
{
    if (!smc->shmem)
        smc->shmem = (void *)msg;
    else {
        u8 result = FIELD_GET(SMC_RESULT_RESULT, msg);
        u8 id = FIELD_GET(SMC_RESULT_ID, msg);
        if (result == SMC_NOTIFICATION) {
            printf("SMC: Notification: 0x%08lx\n", FIELD_GET(SMC_RESULT_VALUE, msg));
            return;
        }
        smc->outstanding[id] = false;
        smc->ret[id] = msg;
    }
}

static int smc_work(smc_dev_t *smc)
{
    int ret;
    struct rtkit_message msg;

    while ((ret = rtkit_recv(smc->rtkit, &msg)) == 0)
        ;

    if (ret < 0) {
        printf("SMC: rtkit_recv failed!\n");
        return ret;
    }

    if (msg.ep != SMC_ENDPOINT) {
        printf("SMC: received message for unexpected endpoint 0x%02x\n", msg.ep);
        return 0;
    }

    smc_handle_msg(smc, msg.msg);

    return 0;
}

static void smc_send(smc_dev_t *smc, u64 message)
{
    struct rtkit_message msg;

    msg.ep = SMC_ENDPOINT;
    msg.msg = message;

    rtkit_send(smc->rtkit, &msg);
}

static int smc_cmd(smc_dev_t *smc, u64 message)
{
    u8 id = smc->msgid++ & 0xF;
    assert(!smc->outstanding[id]);
    smc->outstanding[id] = true;

    message |= FIELD_PREP(SMC_MSG_ID, id);

    smc_send(smc, message);
    while (smc->outstanding[id])
        smc_work(smc);

    u64 result = smc->ret[id];
    u32 ret = FIELD_GET(SMC_RESULT_RESULT, result);
    if (ret) {
        printf("SMC: smc_cmd[0x%x] failed: %u\n", id, ret);
        return ret;
    }

    return 0;
}

void smc_shutdown(smc_dev_t *smc)
{
    rtkit_quiesce(smc->rtkit);
    rtkit_free(smc->rtkit);
    asc_free(smc->asc);
    free(smc);
}

smc_dev_t *smc_init(void)
{
    smc_dev_t *smc = calloc(1, sizeof(smc_dev_t));
    if (!smc)
        return NULL;

    smc->asc = asc_init("/arm-io/smc");
    if (!smc->asc) {
        printf("SMC: failed to initialize ASC\n");
        goto out_free;
    }

    smc->rtkit = rtkit_init("smc", smc->asc, NULL, NULL, NULL, true);
    if (!smc->rtkit) {
        printf("SMC: failed to initialize RTKit\n");
        goto out_asc;
    }

    if (!rtkit_boot(smc->rtkit)) {
        printf("SMC: failed to boot RTKit\n");
        goto out_rtkit;
    }

    if (!rtkit_start_ep(smc->rtkit, SMC_ENDPOINT)) {
        printf("SMC: failed start SMC endpoint\n");
        goto out_rtkit;
    }

    u64 initialize =
        FIELD_PREP(SMC_MSG_TYPE, SMC_INITIALIZE) | FIELD_PREP(SMC_MSG_ID, smc->msgid++);

    smc_send(smc, initialize);

    while (!smc->shmem) {
        int ret = smc_work(smc);
        if (ret < 0)
            goto out_rtkit;
    }

    return smc;

out_rtkit:
    rtkit_free(smc->rtkit);
out_asc:
    asc_free(smc->asc);
out_free:
    free(smc);
    return NULL;
}

int smc_write_u32(smc_dev_t *smc, u32 key, u32 value)
{
    memcpy(smc->shmem, &value, sizeof(value));
    u64 msg = FIELD_PREP(SMC_MSG_TYPE, SMC_WRITE_KEY);
    msg |= FIELD_PREP(SMC_WRITE_KEY_SIZE, sizeof(value));
    msg |= FIELD_PREP(SMC_WRITE_KEY_KEY, key);

    return smc_cmd(smc, msg);
}

#define SMC_NUDGE_ID   0xF
#define SMC_NUDGE_KEY  0x234b4559 /* '#KEY' big-endian — fourcc */

int smc_nudge(smc_dev_t *smc)
{
    if (!smc || !smc->rtkit || !smc->asc)
        return -1;

    /* Drain any replies waiting for us first. rtkit_recv is
     * non-blocking. If the previous nudge has come back, its
     * handler (smc_handle_msg) will clear outstanding[NUDGE_ID]. */
    struct rtkit_message dmsg;
    for (int guard = 0; guard < 8; guard++) {
        int r = rtkit_recv(smc->rtkit, &dmsg);
        if (r <= 0)
            break;
        if (dmsg.ep == SMC_ENDPOINT)
            smc_handle_msg(smc, dmsg.msg);
    }

    /* If the previous nudge is still outstanding, don't pile up. */
    if (smc->outstanding[SMC_NUDGE_ID])
        return 0;

    /* Don't block if the A2I mailbox is full. */
    if (!asc_can_send(smc->asc))
        return 0;

    /* Build SMC_READ_KEY(#KEY). Harmless, read-only, always-exists. */
    u64 msg = FIELD_PREP(SMC_MSG_TYPE, SMC_READ_KEY)
            | FIELD_PREP(SMC_MSG_ID, SMC_NUDGE_ID)
            | FIELD_PREP(SMC_WRITE_KEY_KEY, SMC_NUDGE_KEY);

    smc->outstanding[SMC_NUDGE_ID] = true;

    struct asc_message amsg;
    amsg.msg0 = msg;
    amsg.msg1 = SMC_ENDPOINT;
    if (!asc_send(smc->asc, &amsg)) {
        /* asc_send may have polled the FULL bit for up to 200ms
         * before failing. Unlikely but possible. Roll back state. */
        smc->outstanding[SMC_NUDGE_ID] = false;
        return -1;
    }
    return 1;
}

int smc_pump(smc_dev_t *smc)
{
    if (!smc || !smc->rtkit)
        return 0;

    struct rtkit_message msg;
    int drained = 0;
    /* rtkit_recv() returns 1 when it handed us an app-level msg,
     * 0 when it consumed an infra-level msg (syslog / ioreport /
     * mgmt) on our behalf, and <0 on fatal error. Drain whichever
     * are queued. Non-blocking: each call consumes at most one
     * asc_recv or returns 0 if the I2A mailbox is empty. */
    for (int guard = 0; guard < 16; guard++) {
        int r = rtkit_recv(smc->rtkit, &msg);
        if (r <= 0)
            break;
        /* If we ever start caring about app-level messages (ep
         * 0x20 SMC NOTIFICATIONs etc) we'd demux here. For the
         * keep-alive probe we just want rtkit to have touched
         * the mailbox, so treat the message as drained. */
        if (msg.ep == SMC_ENDPOINT)
            smc_handle_msg(smc, msg.msg);
        drained++;
    }
    return drained;
}
