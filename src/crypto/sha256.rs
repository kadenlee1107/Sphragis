// Bat_OS — SHA-256 Implementation
// Pure Rust, zero dependencies. Used for Merkle tree and key derivation.

const K: [u32; 64] = [
    0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
    0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
    0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
    0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
    0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
    0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
    0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
    0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2,
];

#[derive(Clone)]
pub struct Sha256 {
    state: [u32; 8],
    buffer: [u8; 64],
    buf_len: usize,
    total_len: u64,
}

impl Sha256 {
    pub fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
            ],
            buffer: [0; 64],
            buf_len: 0,
            total_len: 0,
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        let mut offset = 0;

        // Fill buffer if partially full
        if self.buf_len > 0 {
            let space = 64 - self.buf_len;
            let take = data.len().min(space);
            self.buffer[self.buf_len..self.buf_len + take].copy_from_slice(&data[..take]);
            self.buf_len += take;
            offset += take;

            if self.buf_len == 64 {
                let block = self.buffer;
                self.process_block(&block);
                self.buf_len = 0;
            }
        }

        // Process full blocks
        while offset + 64 <= data.len() {
            let mut block = [0u8; 64];
            block.copy_from_slice(&data[offset..offset + 64]);
            self.process_block(&block);
            offset += 64;
        }

        // Buffer remainder
        if offset < data.len() {
            let remaining = data.len() - offset;
            self.buffer[..remaining].copy_from_slice(&data[offset..]);
            self.buf_len = remaining;
        }

        self.total_len += data.len() as u64;
    }

    pub fn finalize(mut self) -> [u8; 32] {
        let bit_len = self.total_len * 8;

        // Padding
        self.buffer[self.buf_len] = 0x80;
        self.buf_len += 1;

        if self.buf_len > 56 {
            // Need extra block
            while self.buf_len < 64 {
                self.buffer[self.buf_len] = 0;
                self.buf_len += 1;
            }
            let block = self.buffer;
            self.process_block(&block);
            self.buf_len = 0;
            self.buffer = [0; 64];
        }

        while self.buf_len < 56 {
            self.buffer[self.buf_len] = 0;
            self.buf_len += 1;
        }

        // Append length in bits (big-endian)
        self.buffer[56..64].copy_from_slice(&bit_len.to_be_bytes());
        let block = self.buffer;
        self.process_block(&block);

        // Output
        let mut hash = [0u8; 32];
        for i in 0..8 {
            hash[i * 4..(i + 1) * 4].copy_from_slice(&self.state[i].to_be_bytes());
        }
        hash
    }

    fn process_block(&mut self, block: &[u8; 64]) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block[i * 4],
                block[i * 4 + 1],
                block[i * 4 + 2],
                block[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
            let s1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19) ^ (w[i-2] >> 10);
            w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);

            h = g; g = f; f = e;
            e = d.wrapping_add(t1);
            d = c; c = b; b = a;
            a = t1.wrapping_add(t2);
        }

        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }
}

/// Hash a byte slice and return the 32-byte digest.
pub fn hash(data: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(data);
    h.finalize()
}

/// Simple key derivation: HMAC-like construction.
pub fn derive_key(master: &[u8; 32], context: &[u8]) -> [u8; 32] {
    hmac(master, context)
}

/// HMAC-SHA256(key, message) → 32 bytes
pub fn hmac(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut padded_key = [0u8; 64];
    if key.len() > 64 {
        let h = hash(key);
        padded_key[..32].copy_from_slice(&h);
    } else {
        padded_key[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= padded_key[i];
        opad[i] ^= padded_key[i];
    }

    let mut inner = Sha256::new();
    inner.update(&ipad);
    inner.update(message);
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(&opad);
    outer.update(&inner_hash);
    outer.finalize()
}

/// HKDF-Extract(salt, ikm) → PRK (32 bytes)
pub fn hkdf_extract(salt: &[u8], ikm: &[u8]) -> [u8; 32] {
    let s = if salt.is_empty() { &[0u8; 32] as &[u8] } else { salt };
    hmac(s, ikm)
}

/// HKDF-Expand(prk, info, length) → OKM
/// Only supports length <= 32 (one block for SHA-256)
pub fn hkdf_expand(prk: &[u8; 32], info: &[u8], length: usize) -> [u8; 32] {
    // T(1) = HMAC-Hash(PRK, info || 0x01)
    let mut input = [0u8; 256];
    let ilen = info.len().min(254);
    input[..ilen].copy_from_slice(&info[..ilen]);
    input[ilen] = 0x01;
    let result = hmac(prk, &input[..ilen + 1]);

    if length <= 32 {
        return result;
    }
    // For length > 32, would need T(2) etc. Not needed for TLS 1.3 key derivation
    result
}

/// HKDF-Expand-Label for TLS 1.3
/// Derives keys per RFC 8446 Section 7.1
pub fn hkdf_expand_label(secret: &[u8; 32], label: &[u8], context: &[u8], length: usize) -> [u8; 32] {
    // HkdfLabel = length(2) || "tls13 " || label || context
    let mut info = [0u8; 128];
    let mut pos = 0;

    // length (2 bytes, big-endian)
    info[pos] = (length >> 8) as u8; pos += 1;
    info[pos] = length as u8; pos += 1;

    // label with "tls13 " prefix
    let prefix = b"tls13 ";
    let label_len = prefix.len() + label.len();
    info[pos] = label_len as u8; pos += 1;
    info[pos..pos + prefix.len()].copy_from_slice(prefix); pos += prefix.len();
    let ll = label.len().min(64);
    info[pos..pos + ll].copy_from_slice(&label[..ll]); pos += ll;

    // context
    let cl = context.len().min(32);
    info[pos] = cl as u8; pos += 1;
    if cl > 0 {
        info[pos..pos + cl].copy_from_slice(&context[..cl]); pos += cl;
    }

    hkdf_expand(secret, &info[..pos], length)
}
