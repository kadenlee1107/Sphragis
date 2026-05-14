#![allow(dead_code)]
// Sphragis — Inter-Process Communication
// Synchronous message passing between tasks.
// Capability-checked: you can only send/receive on channels you hold caps for.

use crate::kernel::process::{self, TaskId, TaskState};
use crate::kernel::capability::CapType;
use crate::drivers::uart;

pub const MAX_CHANNELS: usize = 64;
const MAX_MSG_SIZE: usize = 256;

#[derive(Clone, Copy)]
pub struct Message {
    pub sender: u16,
    pub msg_type: u32,
    pub data: [u8; MAX_MSG_SIZE],
    pub len: usize,
}

impl Message {
    pub const fn empty() -> Self {
        Self {
            sender: 0,
            msg_type: 0,
            data: [0u8; MAX_MSG_SIZE],
            len: 0,
        }
    }

    pub fn new(msg_type: u32, payload: &[u8]) -> Self {
        let mut msg = Self::empty();
        msg.msg_type = msg_type;
        let copy_len = payload.len().min(MAX_MSG_SIZE);
        msg.data[..copy_len].copy_from_slice(&payload[..copy_len]);
        msg.len = copy_len;
        msg
    }

    pub fn payload(&self) -> &[u8] {
        &self.data[..self.len]
    }
}

struct Channel {
    active: bool,
    buffer: Option<Message>,
    waiting_sender: Option<TaskId>,
    waiting_receiver: Option<TaskId>,
}

impl Channel {
    const fn empty() -> Self {
        Self {
            active: false,
            buffer: None,
            waiting_sender: None,
            waiting_receiver: None,
        }
    }
}

static mut CHANNELS: [Channel; MAX_CHANNELS] = {
    const EMPTY: Channel = Channel::empty();
    [EMPTY; MAX_CHANNELS]
};

pub fn init() {
    uart::puts("  [ipc] IPC channels initialized\n");
}

/// Create a new IPC channel. Returns the channel ID.
pub fn create_channel() -> Option<u64> {
    unsafe {
        for i in 0..MAX_CHANNELS {
            if !CHANNELS[i].active {
                CHANNELS[i].active = true;
                CHANNELS[i].buffer = None;
                CHANNELS[i].waiting_sender = None;
                CHANNELS[i].waiting_receiver = None;
                return Some(i as u64);
            }
        }
    }
    None
}

/// Send a message on a channel (synchronous — blocks until received).
pub fn send(channel_id: u64, mut msg: Message) -> Result<(), &'static str> {
    let ch = channel_id as usize;
    if ch >= MAX_CHANNELS {
        return Err("invalid channel");
    }

    let sender = process::current_id();

    // Capability check: sender must hold IpcSend for this channel
    let task = process::get(sender);
    if !task.capabilities.has(CapType::IpcSend, channel_id) {
        return Err("no send capability");
    }

    msg.sender = sender.0;

    unsafe {
        if !CHANNELS[ch].active {
            return Err("channel not active");
        }

        // If a receiver is already waiting, deliver immediately
        if let Some(recv_id) = CHANNELS[ch].waiting_receiver.take() {
            CHANNELS[ch].buffer = Some(msg);
            let receiver = process::get(recv_id);
            receiver.state = TaskState::Ready;
            return Ok(());
        }

        // Otherwise, block sender until a receiver arrives
        CHANNELS[ch].buffer = Some(msg);
        CHANNELS[ch].waiting_sender = Some(sender);
        let current = process::get(sender);
        current.state = TaskState::Blocked;
        crate::kernel::scheduler::yield_now();
    }

    Ok(())
}

/// Receive a message from a channel (synchronous — blocks until message arrives).
pub fn recv(channel_id: u64) -> Result<Message, &'static str> {
    let ch = channel_id as usize;
    if ch >= MAX_CHANNELS {
        return Err("invalid channel");
    }

    let receiver = process::current_id();

    // Capability check: receiver must hold IpcRecv for this channel
    let task = process::get(receiver);
    if !task.capabilities.has(CapType::IpcRecv, channel_id) {
        return Err("no recv capability");
    }

    unsafe {
        if !CHANNELS[ch].active {
            return Err("channel not active");
        }

        // If there's a message buffered, take it
        if let Some(msg) = CHANNELS[ch].buffer.take() {
            // Unblock sender if one is waiting
            if let Some(send_id) = CHANNELS[ch].waiting_sender.take() {
                let sender = process::get(send_id);
                sender.state = TaskState::Ready;
            }
            return Ok(msg);
        }

        // Otherwise, block receiver until a sender arrives
        CHANNELS[ch].waiting_receiver = Some(receiver);
        let current = process::get(receiver);
        current.state = TaskState::Blocked;
        crate::kernel::scheduler::yield_now();

        // When we wake up, retry — loop until message arrives
        loop {
            if let Some(msg) = CHANNELS[ch].buffer.take() {
                if let Some(send_id) = CHANNELS[ch].waiting_sender.take() {
                    let sender = process::get(send_id);
                    sender.state = TaskState::Ready;
                }
                return Ok(msg);
            }
            // Not ready yet, block again
            CHANNELS[ch].waiting_receiver = Some(receiver);
            let current = process::get(receiver);
            current.state = TaskState::Blocked;
            crate::kernel::scheduler::yield_now();
        }
    }
}
