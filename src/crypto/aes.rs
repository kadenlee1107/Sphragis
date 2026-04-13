// Bat_OS — AES-256 Implementation
// Pure Rust, zero dependencies. Full AES-256 in ECB/CTR modes.
// Every constant is defined here — no imports, no trust.

const NB: usize = 4;  // Block size in 32-bit words (128 bits)
const NK: usize = 8;  // Key length in 32-bit words (256 bits)
const NR: usize = 14; // Number of rounds for AES-256

// AES S-Box
static SBOX: [u8; 256] = [
    0x63,0x7c,0x77,0x7b,0xf2,0x6b,0x6f,0xc5,0x30,0x01,0x67,0x2b,0xfe,0xd7,0xab,0x76,
    0xca,0x82,0xc9,0x7d,0xfa,0x59,0x47,0xf0,0xad,0xd4,0xa2,0xaf,0x9c,0xa4,0x72,0xc0,
    0xb7,0xfd,0x93,0x26,0x36,0x3f,0xf7,0xcc,0x34,0xa5,0xe5,0xf1,0x71,0xd8,0x31,0x15,
    0x04,0xc7,0x23,0xc3,0x18,0x96,0x05,0x9a,0x07,0x12,0x80,0xe2,0xeb,0x27,0xb2,0x75,
    0x09,0x83,0x2c,0x1a,0x1b,0x6e,0x5a,0xa0,0x52,0x3b,0xd6,0xb3,0x29,0xe3,0x2f,0x84,
    0x53,0xd1,0x00,0xed,0x20,0xfc,0xb1,0x5b,0x6a,0xcb,0xbe,0x39,0x4a,0x4c,0x58,0xcf,
    0xd0,0xef,0xaa,0xfb,0x43,0x4d,0x33,0x85,0x45,0xf9,0x02,0x7f,0x50,0x3c,0x9f,0xa8,
    0x51,0xa3,0x40,0x8f,0x92,0x9d,0x38,0xf5,0xbc,0xb6,0xda,0x21,0x10,0xff,0xf3,0xd2,
    0xcd,0x0c,0x13,0xec,0x5f,0x97,0x44,0x17,0xc4,0xa7,0x7e,0x3d,0x64,0x5d,0x19,0x73,
    0x60,0x81,0x4f,0xdc,0x22,0x2a,0x90,0x88,0x46,0xee,0xb8,0x14,0xde,0x5e,0x0b,0xdb,
    0xe0,0x32,0x3a,0x0a,0x49,0x06,0x24,0x5c,0xc2,0xd3,0xac,0x62,0x91,0x95,0xe4,0x79,
    0xe7,0xc8,0x37,0x6d,0x8d,0xd5,0x4e,0xa9,0x6c,0x56,0xf4,0xea,0x65,0x7a,0xae,0x08,
    0xba,0x78,0x25,0x2e,0x1c,0xa6,0xb4,0xc6,0xe8,0xdd,0x74,0x1f,0x4b,0xbd,0x8b,0x8a,
    0x70,0x3e,0xb5,0x66,0x48,0x03,0xf6,0x0e,0x61,0x35,0x57,0xb9,0x86,0xc1,0x1d,0x9e,
    0xe1,0xf8,0x98,0x11,0x69,0xd9,0x8e,0x94,0x9b,0x1e,0x87,0xe9,0xce,0x55,0x28,0xdf,
    0x8c,0xa1,0x89,0x0d,0xbf,0xe6,0x42,0x68,0x41,0x99,0x2d,0x0f,0xb0,0x54,0xbb,0x16,
];

// Round constants
static RCON: [u8; 11] = [0x00,0x01,0x02,0x04,0x08,0x10,0x20,0x40,0x80,0x1b,0x36];

pub struct Aes256 {
    round_keys: [u32; 4 * (NR + 1)],
}

impl Aes256 {
    pub fn new(key: &[u8; 32]) -> Self {
        let mut rk = [0u32; 4 * (NR + 1)];
        key_expansion(key, &mut rk);
        Self { round_keys: rk }
    }

    /// Encrypt a single 16-byte block in place.
    pub fn encrypt_block(&self, block: &mut [u8; 16]) {
        let mut state = [[0u8; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                state[j][i] = block[i * 4 + j];
            }
        }

        add_round_key(&mut state, &self.round_keys, 0);

        for round in 1..NR {
            sub_bytes(&mut state);
            shift_rows(&mut state);
            mix_columns(&mut state);
            add_round_key(&mut state, &self.round_keys, round);
        }

        sub_bytes(&mut state);
        shift_rows(&mut state);
        add_round_key(&mut state, &self.round_keys, NR);

        for i in 0..4 {
            for j in 0..4 {
                block[i * 4 + j] = state[j][i];
            }
        }
    }

    /// Encrypt data using CTR mode (supports arbitrary length).
    /// CTR mode: encrypt a counter, XOR with plaintext.
    /// Same function for encrypt and decrypt.
    pub fn ctr_crypt(&self, nonce: &[u8; 12], data: &mut [u8]) {
        let mut counter = [0u8; 16];
        counter[..12].copy_from_slice(nonce);

        let mut block_num: u32 = 0;
        let mut offset = 0;

        while offset < data.len() {
            // Set counter value (big-endian)
            counter[12] = (block_num >> 24) as u8;
            counter[13] = (block_num >> 16) as u8;
            counter[14] = (block_num >> 8) as u8;
            counter[15] = block_num as u8;

            // Encrypt counter to get keystream
            let mut keystream = counter;
            self.encrypt_block(&mut keystream);

            // XOR keystream with data
            let remaining = data.len() - offset;
            let chunk = if remaining < 16 { remaining } else { 16 };
            for i in 0..chunk {
                data[offset + i] ^= keystream[i];
            }

            offset += 16;
            block_num += 1;
        }
    }
}

fn key_expansion(key: &[u8; 32], rk: &mut [u32; 4 * (NR + 1)]) {
    for i in 0..NK {
        rk[i] = u32::from_be_bytes([
            key[4 * i],
            key[4 * i + 1],
            key[4 * i + 2],
            key[4 * i + 3],
        ]);
    }

    for i in NK..(4 * (NR + 1)) {
        let mut temp = rk[i - 1];
        if i % NK == 0 {
            temp = sub_word(rot_word(temp)) ^ ((RCON[i / NK] as u32) << 24);
        } else if i % NK == 4 {
            temp = sub_word(temp);
        }
        rk[i] = rk[i - NK] ^ temp;
    }
}

fn sub_word(w: u32) -> u32 {
    let b = w.to_be_bytes();
    u32::from_be_bytes([
        SBOX[b[0] as usize],
        SBOX[b[1] as usize],
        SBOX[b[2] as usize],
        SBOX[b[3] as usize],
    ])
}

fn rot_word(w: u32) -> u32 {
    (w << 8) | (w >> 24)
}

fn sub_bytes(state: &mut [[u8; 4]; 4]) {
    for i in 0..4 {
        for j in 0..4 {
            state[i][j] = SBOX[state[i][j] as usize];
        }
    }
}

fn shift_rows(state: &mut [[u8; 4]; 4]) {
    // Row 1: shift left 1
    let tmp = state[1][0];
    state[1][0] = state[1][1];
    state[1][1] = state[1][2];
    state[1][2] = state[1][3];
    state[1][3] = tmp;

    // Row 2: shift left 2
    let (t0, t1) = (state[2][0], state[2][1]);
    state[2][0] = state[2][2];
    state[2][1] = state[2][3];
    state[2][2] = t0;
    state[2][3] = t1;

    // Row 3: shift left 3 (= shift right 1)
    let tmp = state[3][3];
    state[3][3] = state[3][2];
    state[3][2] = state[3][1];
    state[3][1] = state[3][0];
    state[3][0] = tmp;
}

fn mix_columns(state: &mut [[u8; 4]; 4]) {
    for i in 0..4 {
        let a = state[0][i];
        let b = state[1][i];
        let c = state[2][i];
        let d = state[3][i];

        state[0][i] = gmul(2, a) ^ gmul(3, b) ^ c ^ d;
        state[1][i] = a ^ gmul(2, b) ^ gmul(3, c) ^ d;
        state[2][i] = a ^ b ^ gmul(2, c) ^ gmul(3, d);
        state[3][i] = gmul(3, a) ^ b ^ c ^ gmul(2, d);
    }
}

fn add_round_key(state: &mut [[u8; 4]; 4], rk: &[u32], round: usize) {
    for i in 0..4 {
        let k = rk[round * 4 + i].to_be_bytes();
        for j in 0..4 {
            state[j][i] ^= k[j];
        }
    }
}

/// Galois field multiplication
fn gmul(mut a: u8, mut b: u8) -> u8 {
    let mut p: u8 = 0;
    for _ in 0..8 {
        if b & 1 != 0 {
            p ^= a;
        }
        let hi = a & 0x80;
        a <<= 1;
        if hi != 0 {
            a ^= 0x1b; // x^8 + x^4 + x^3 + x + 1
        }
        b >>= 1;
    }
    p
}
