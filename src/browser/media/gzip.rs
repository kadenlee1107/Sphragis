#![allow(dead_code)]
// Bat_OS — Gzip Decompressor (RFC 1952)
// Parses gzip headers and decompresses the DEFLATE payload.
// Implements full DEFLATE (RFC 1951): stored, fixed Huffman, dynamic Huffman.
//
// Gzip format:
//   [10-byte header: ID1(0x1f) ID2(0x8b) CM(8) FLG MTIME(4) XFL OS]
//   [optional extras based on FLG bits]
//   [DEFLATE compressed data]
//   [CRC32(4) ISIZE(4)]

/// Decompress a gzip-compressed buffer.
/// Returns the number of decompressed bytes written to `output`.
/// Returns 0 on error (bad header, decompression failure, etc.).
pub fn decompress(input: &[u8], output: &mut [u8]) -> usize {
    // Minimum gzip: 10-byte header + 8-byte trailer = 18 bytes
    if input.len() < 18 {
        return 0;
    }

    // Verify gzip magic bytes and compression method
    if input[0] != 0x1f || input[1] != 0x8b {
        return 0; // not gzip
    }
    if input[2] != 8 {
        return 0; // only DEFLATE (method 8) supported
    }

    let flg = input[3];
    let mut pos: usize = 10; // skip fixed header

    // FLG bit 2 (FEXTRA): skip extra field
    if flg & 0x04 != 0 {
        if pos + 2 > input.len() { return 0; }
        let xlen = (input[pos] as usize) | ((input[pos + 1] as usize) << 8);
        pos += 2 + xlen;
    }

    // FLG bit 3 (FNAME): skip null-terminated original file name
    if flg & 0x08 != 0 {
        while pos < input.len() && input[pos] != 0 {
            pos += 1;
        }
        pos += 1; // skip the null terminator
    }

    // FLG bit 4 (FCOMMENT): skip null-terminated comment
    if flg & 0x10 != 0 {
        while pos < input.len() && input[pos] != 0 {
            pos += 1;
        }
        pos += 1; // skip the null terminator
    }

    // FLG bit 1 (FHCRC): skip 2-byte header CRC16
    if flg & 0x02 != 0 {
        pos += 2;
    }

    if pos >= input.len() { return 0; }

    // The remaining data (minus the 8-byte trailer) is the DEFLATE stream
    let deflate_end = if input.len() > 8 { input.len() - 8 } else { input.len() };
    let deflate_data = &input[pos..deflate_end];

    crate::drivers::uart::puts("[gzip] deflate data at offset ");
    crate::kernel::mm::print_num(pos);
    crate::drivers::uart::puts(", ");
    crate::kernel::mm::print_num(deflate_data.len());
    crate::drivers::uart::puts(" bytes, first bytes: ");
    for i in 0..deflate_data.len().min(8) {
        let hex = b"0123456789abcdef";
        crate::drivers::uart::putc(hex[(deflate_data[i] >> 4) as usize]);
        crate::drivers::uart::putc(hex[(deflate_data[i] & 0xf) as usize]);
        crate::drivers::uart::putc(b' ');
    }
    crate::drivers::uart::puts("\n");

    // Use the PROVEN inflate implementation from the PNG decoder
    match super::png::inflate(deflate_data, output) {
        Ok(n) => {
            crate::drivers::uart::puts("[gzip] decompressed ");
            crate::kernel::mm::print_num(n);
            crate::drivers::uart::puts(" bytes\n");
            n
        }
        Err(e) => {
            crate::drivers::uart::puts("[gzip] inflate error: ");
            crate::drivers::uart::puts(e);
            crate::drivers::uart::puts("\n");
            0
        }
    }
}

// ─── DEFLATE Decompression (RFC 1951) ───
// Standalone implementation — same algorithm as in png.rs but self-contained.

fn inflate(input: &[u8], output: &mut [u8]) -> Result<usize, &'static str> {
    let mut reader = BitReader::new(input);
    let mut out_pos = 0usize;

    loop {
        let bfinal = reader.read_bits(1)?;
        let btype = reader.read_bits(2)?;

        match btype {
            0 => {
                // Stored (uncompressed) block
                reader.align_byte();
                let len = reader.read_bits(16)? as usize;
                let _nlen = reader.read_bits(16)?;
                for _ in 0..len {
                    if out_pos >= output.len() { break; }
                    output[out_pos] = reader.read_byte()?;
                    out_pos += 1;
                }
            }
            1 => {
                // Fixed Huffman codes
                inflate_block_fixed(&mut reader, output, &mut out_pos)?;
            }
            2 => {
                // Dynamic Huffman codes
                inflate_block_dynamic(&mut reader, output, &mut out_pos)?;
            }
            _ => return Err("invalid block type"),
        }

        if bfinal == 1 { break; }
    }

    Ok(out_pos)
}

/// Inflate a block with fixed Huffman codes
fn inflate_block_fixed(reader: &mut BitReader, output: &mut [u8], out_pos: &mut usize) -> Result<(), &'static str> {
    loop {
        let code = decode_fixed_literal(reader)?;

        if code < 256 {
            if *out_pos < output.len() {
                output[*out_pos] = code as u8;
                *out_pos += 1;
            }
        } else if code == 256 {
            return Ok(());
        } else {
            let length = decode_length(code, reader)?;
            let dist_code = read_reversed_bits(reader, 5)?;
            let distance = decode_distance(dist_code, reader)?;

            for _ in 0..length {
                if *out_pos < output.len() && *out_pos >= distance {
                    output[*out_pos] = output[*out_pos - distance];
                    *out_pos += 1;
                }
            }
        }
    }
}

/// Inflate a block with dynamic Huffman codes
fn inflate_block_dynamic(reader: &mut BitReader, output: &mut [u8], out_pos: &mut usize) -> Result<(), &'static str> {
    let hlit = reader.read_bits(5)? as usize + 257;
    let hdist = reader.read_bits(5)? as usize + 1;
    let hclen = reader.read_bits(4)? as usize + 4;

    // Read code length code lengths
    let cl_order = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
    let mut cl_lens = [0u8; 19];
    for i in 0..hclen {
        cl_lens[cl_order[i]] = reader.read_bits(3)? as u8;
    }

    // Build code length Huffman table
    let cl_table = build_huffman_table(&cl_lens, 19);

    // Read literal/length + distance code lengths
    let mut all_lens = [0u8; 320]; // hlit + hdist
    let total = hlit + hdist;
    let mut i = 0;
    while i < total {
        let sym = decode_huffman(reader, &cl_table)?;
        match sym {
            0..=15 => {
                all_lens[i] = sym as u8;
                i += 1;
            }
            16 => {
                let repeat = reader.read_bits(2)? as usize + 3;
                let prev = if i > 0 { all_lens[i - 1] } else { 0 };
                for _ in 0..repeat { if i < total { all_lens[i] = prev; i += 1; } }
            }
            17 => {
                let repeat = reader.read_bits(3)? as usize + 3;
                for _ in 0..repeat { if i < total { all_lens[i] = 0; i += 1; } }
            }
            18 => {
                let repeat = reader.read_bits(7)? as usize + 11;
                for _ in 0..repeat { if i < total { all_lens[i] = 0; i += 1; } }
            }
            _ => return Err("invalid code length code"),
        }
    }

    // Build literal/length and distance Huffman tables
    let lit_table = build_huffman_table(&all_lens[..hlit], hlit);
    let dist_table = build_huffman_table(&all_lens[hlit..hlit + hdist], hdist);

    // Decode data
    loop {
        let sym = decode_huffman(reader, &lit_table)?;

        if sym < 256 {
            if *out_pos < output.len() {
                output[*out_pos] = sym as u8;
                *out_pos += 1;
            }
        } else if sym == 256 {
            return Ok(());
        } else {
            let length = decode_length(sym, reader)?;
            let dist_sym = decode_huffman(reader, &dist_table)?;
            let distance = decode_distance(dist_sym, reader)?;

            for _ in 0..length {
                if *out_pos < output.len() && *out_pos >= distance {
                    output[*out_pos] = output[*out_pos - distance];
                    *out_pos += 1;
                }
            }
        }
    }
}

// ─── Huffman Tables ───

const MAX_HUFFMAN: usize = 320;
const MAX_BITS: usize = 15;

struct HuffmanTable {
    symbols: [u16; MAX_HUFFMAN],
    counts: [u16; MAX_BITS + 1],
    offsets: [u16; MAX_BITS + 1],
}

fn build_huffman_table(lens: &[u8], count: usize) -> HuffmanTable {
    let mut table = HuffmanTable {
        symbols: [0; MAX_HUFFMAN],
        counts: [0; MAX_BITS + 1],
        offsets: [0; MAX_BITS + 1],
    };

    for i in 0..count.min(MAX_HUFFMAN) {
        let l = lens[i] as usize;
        if l <= MAX_BITS { table.counts[l] += 1; }
    }
    table.counts[0] = 0;

    let mut offset = 0u16;
    for i in 1..=MAX_BITS {
        table.offsets[i] = offset;
        offset += table.counts[i];
    }

    for i in 0..count.min(MAX_HUFFMAN) {
        let l = lens[i] as usize;
        if l > 0 && l <= MAX_BITS {
            let idx = table.offsets[l] as usize;
            if idx < MAX_HUFFMAN {
                table.symbols[idx] = i as u16;
                table.offsets[l] += 1;
            }
        }
    }

    // Restore offsets
    let mut offset = 0u16;
    for i in 1..=MAX_BITS {
        table.offsets[i] = offset;
        offset += table.counts[i];
    }

    table
}

fn decode_huffman(reader: &mut BitReader, table: &HuffmanTable) -> Result<u32, &'static str> {
    let mut code = 0u32;
    let mut first = 0u32;

    for len in 1..=MAX_BITS {
        code = (code << 1) | reader.read_bits(1)?;
        let count = table.counts[len] as u32;
        if code < first + count {
            let idx = (table.offsets[len] as u32 + code - first) as usize;
            if idx < MAX_HUFFMAN {
                return Ok(table.symbols[idx] as u32);
            }
        }
        first = (first + count) << 1;
    }

    Err("invalid huffman code")
}

// ─── DEFLATE fixed code decoding ───

fn decode_fixed_literal(reader: &mut BitReader) -> Result<u32, &'static str> {
    let mut code = read_reversed_bits(reader, 7)?;

    if code <= 23 {
        return Ok(code + 256); // 256-279 (7-bit)
    }
    code = (code << 1) | reader.read_bits(1)?;
    if code >= 48 && code <= 191 {
        return Ok(code - 48); // 0-143
    }
    if code >= 192 && code <= 199 {
        return Ok(code - 192 + 280); // 280-287
    }
    code = (code << 1) | reader.read_bits(1)?;
    if code >= 400 && code <= 511 {
        return Ok(code - 400 + 144); // 144-255
    }

    Err("invalid fixed huffman")
}

fn decode_length(code: u32, reader: &mut BitReader) -> Result<usize, &'static str> {
    let (base, extra_bits) = match code {
        257..=264 => (code - 257 + 3, 0),
        265..=268 => ((code - 265) * 2 + 11, 1),
        269..=272 => ((code - 269) * 4 + 19, 2),
        273..=276 => ((code - 273) * 8 + 35, 3),
        277..=280 => ((code - 277) * 16 + 67, 4),
        281..=284 => ((code - 281) * 32 + 131, 5),
        285 => (258, 0),
        _ => return Err("invalid length code"),
    };
    let extra = if extra_bits > 0 { reader.read_bits(extra_bits as u8)? } else { 0 };
    Ok((base + extra) as usize)
}

fn decode_distance(code: u32, reader: &mut BitReader) -> Result<usize, &'static str> {
    if code <= 3 {
        return Ok((code + 1) as usize);
    }
    let extra_bits = (code / 2 - 1) as u8;
    let base = ((2u32 << (code / 2 - 1)) + 1) + (code % 2) * (1u32 << (code / 2 - 1));
    let extra = reader.read_bits(extra_bits)?;
    Ok((base + extra) as usize)
}

fn read_reversed_bits(reader: &mut BitReader, n: u8) -> Result<u32, &'static str> {
    let mut val = 0u32;
    for _ in 0..n {
        val = (val << 1) | reader.read_bits(1)?;
    }
    Ok(val)
}

// ─── Bit Reader ───

struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
    bit: u8,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        BitReader { data, pos: 0, bit: 0 }
    }

    fn read_bits(&mut self, count: u8) -> Result<u32, &'static str> {
        let mut result = 0u32;
        for i in 0..count {
            if self.pos >= self.data.len() { return Err("unexpected end of data"); }
            let bit_val = (self.data[self.pos] >> self.bit) & 1;
            result |= (bit_val as u32) << i;
            self.bit += 1;
            if self.bit == 8 {
                self.bit = 0;
                self.pos += 1;
            }
        }
        Ok(result)
    }

    fn read_byte(&mut self) -> Result<u8, &'static str> {
        self.read_bits(8).map(|v| v as u8)
    }

    fn align_byte(&mut self) {
        if self.bit > 0 {
            self.bit = 0;
            self.pos += 1;
        }
    }
}
