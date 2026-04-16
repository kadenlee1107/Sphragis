#![allow(dead_code)]
#![allow(unused_assignments)]
// Bat_OS — PNG Decoder
// Decodes PNG images into raw RGBA pixel buffers.
// Implements: PNG chunk parsing, DEFLATE decompression, pixel unfiltering.
//
// PNG format:
//   [8-byte signature] [IHDR chunk] [IDAT chunks...] [IEND chunk]
//   IHDR: width, height, bit_depth, color_type
//   IDAT: DEFLATE-compressed pixel data
//   After decompression: rows of [filter_byte] [pixel_data...]

const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

// Maximum image dimensions (memory constrained)
pub const MAX_WIDTH: usize = 512;
pub const MAX_HEIGHT: usize = 512;
pub const MAX_PIXELS: usize = MAX_WIDTH * MAX_HEIGHT;

/// Decoded PNG image
pub struct PngImage {
    pub width: u32,
    pub height: u32,
    pub pixels: [u32; MAX_PIXELS], // ARGB packed
    pub valid: bool,
}

impl PngImage {
    pub const fn empty() -> Self {
        PngImage {
            width: 0,
            height: 0,
            pixels: [0; MAX_PIXELS],
            valid: false,
        }
    }

    /// Get pixel at (x, y) as ARGB
    pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if x < self.width && y < self.height {
            self.pixels[(y * self.width + x) as usize]
        } else {
            0
        }
    }
}

/// Decode a PNG from raw bytes.
pub fn decode(data: &[u8], image: &mut PngImage) -> Result<(), &'static str> {
    image.valid = false;

    // Check PNG signature
    if data.len() < 8 || data[..8] != PNG_SIGNATURE {
        return Err("not PNG");
    }

    let mut pos = 8;
    let mut width: u32 = 0;
    let mut height: u32 = 0;
    let mut bit_depth: u8 = 0;
    let mut color_type: u8 = 0;

    // Collect all IDAT data
    let mut idat_buf = [0u8; 65536]; // compressed data buffer
    let mut idat_len = 0usize;

    // Parse chunks
    while pos + 12 <= data.len() {
        let chunk_len = u32::from_be_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
        let chunk_type = &data[pos+4..pos+8];
        let chunk_data = &data[pos+8..pos+8+chunk_len.min(data.len()-pos-8)];
        pos += 12 + chunk_len; // skip length(4) + type(4) + data + CRC(4)

        match chunk_type {
            b"IHDR" => {
                if chunk_data.len() >= 13 {
                    width = u32::from_be_bytes([chunk_data[0], chunk_data[1], chunk_data[2], chunk_data[3]]);
                    height = u32::from_be_bytes([chunk_data[4], chunk_data[5], chunk_data[6], chunk_data[7]]);
                    bit_depth = chunk_data[8];
                    color_type = chunk_data[9];
                }
            }
            b"IDAT" => {
                // Accumulate compressed data
                let copy = chunk_data.len().min(idat_buf.len() - idat_len);
                idat_buf[idat_len..idat_len+copy].copy_from_slice(&chunk_data[..copy]);
                idat_len += copy;
            }
            b"IEND" => break,
            _ => {} // skip unknown chunks
        }
    }

    if width == 0 || height == 0 || idat_len == 0 {
        return Err("invalid PNG");
    }
    if width > MAX_WIDTH as u32 || height > MAX_HEIGHT as u32 {
        return Err("PNG too large");
    }

    // Bytes per pixel
    let bpp = match color_type {
        0 => 1,                          // Grayscale
        2 => 3,                          // RGB
        3 => 1,                          // Palette (indexed)
        4 => 2,                          // Grayscale + Alpha
        6 => 4,                          // RGBA
        _ => return Err("unsupported color type"),
    };

    let stride = (width as usize) * bpp + 1; // +1 for filter byte per row
    let decompressed_size = stride * (height as usize);

    // Decompress DEFLATE (skip 2-byte zlib header)
    let zlib_start = if idat_len >= 2 { 2 } else { 0 };
    let mut decompressed = [0u8; 131072]; // max decompressed size
    let decomp_len = inflate(&idat_buf[zlib_start..idat_len], &mut decompressed)?;

    if decomp_len < decompressed_size {
        return Err("incomplete decompression");
    }

    // Unfilter and convert to ARGB pixels
    image.width = width;
    image.height = height;

    let w = width as usize;
    for row in 0..height as usize {
        let row_start = row * stride;
        let filter = decompressed[row_start];
        let pixels_start = row_start + 1;

        // Apply PNG filter
        match filter {
            0 => {} // None — raw bytes
            1 => {  // Sub — difference from left pixel
                for x in bpp..w*bpp {
                    decompressed[pixels_start + x] =
                        decompressed[pixels_start + x].wrapping_add(decompressed[pixels_start + x - bpp]);
                }
            }
            2 => {  // Up — difference from above pixel
                if row > 0 {
                    let prev_start = (row - 1) * stride + 1;
                    for x in 0..w*bpp {
                        decompressed[pixels_start + x] =
                            decompressed[pixels_start + x].wrapping_add(decompressed[prev_start + x]);
                    }
                }
            }
            3 => {  // Average — average of left and above
                let prev_start = if row > 0 { (row - 1) * stride + 1 } else { 0 };
                for x in 0..w*bpp {
                    let left = if x >= bpp { decompressed[pixels_start + x - bpp] as u16 } else { 0 };
                    let above = if row > 0 { decompressed[prev_start + x] as u16 } else { 0 };
                    decompressed[pixels_start + x] =
                        decompressed[pixels_start + x].wrapping_add(((left + above) / 2) as u8);
                }
            }
            4 => {  // Paeth predictor
                let prev_start = if row > 0 { (row - 1) * stride + 1 } else { 0 };
                for x in 0..w*bpp {
                    let left = if x >= bpp { decompressed[pixels_start + x - bpp] as i32 } else { 0 };
                    let above = if row > 0 { decompressed[prev_start + x] as i32 } else { 0 };
                    let upper_left = if row > 0 && x >= bpp { decompressed[prev_start + x - bpp] as i32 } else { 0 };
                    decompressed[pixels_start + x] =
                        decompressed[pixels_start + x].wrapping_add(paeth(left, above, upper_left) as u8);
                }
            }
            _ => {} // Unknown filter — leave as is
        }

        // Convert to ARGB
        for x in 0..w {
            let px = pixels_start + x * bpp;
            let argb = match color_type {
                0 => { // Grayscale
                    let g = decompressed[px];
                    0xFF000000 | (g as u32) << 16 | (g as u32) << 8 | g as u32
                }
                2 => { // RGB
                    let r = decompressed[px];
                    let g = decompressed[px+1];
                    let b = decompressed[px+2];
                    0xFF000000 | (b as u32) << 16 | (g as u32) << 8 | r as u32
                }
                4 => { // Grayscale + Alpha
                    let g = decompressed[px];
                    let a = decompressed[px+1];
                    (a as u32) << 24 | (g as u32) << 16 | (g as u32) << 8 | g as u32
                }
                6 => { // RGBA
                    let r = decompressed[px];
                    let g = decompressed[px+1];
                    let b = decompressed[px+2];
                    let a = decompressed[px+3];
                    (a as u32) << 24 | (b as u32) << 16 | (g as u32) << 8 | r as u32
                }
                _ => 0xFF000000,
            };
            if row * w + x < MAX_PIXELS {
                image.pixels[row * w + x] = argb;
            }
        }
    }

    image.valid = true;
    Ok(())
}

/// Paeth predictor function
fn paeth(a: i32, b: i32, c: i32) -> i32 {
    let p = a + b - c;
    let pa = (p - a).abs();
    let pb = (p - b).abs();
    let pc = (p - c).abs();
    if pa <= pb && pa <= pc { a }
    else if pb <= pc { b }
    else { c }
}

// ─── DEFLATE Decompression (RFC 1951) ───

/// Decompress DEFLATE-compressed data.
pub fn inflate(input: &[u8], output: &mut [u8]) -> Result<usize, &'static str> {
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
        // Read literal/length code (7-9 bits)
        let code = decode_fixed_literal(reader)?;

        if code < 256 {
            // Literal byte
            if *out_pos < output.len() {
                output[*out_pos] = code as u8;
                *out_pos += 1;
            }
        } else if code == 256 {
            // End of block
            return Ok(());
        } else {
            // Length + distance
            let length = decode_length(code, reader)?;
            let dist_code = read_reversed_bits(reader, 5)?;
            let distance = decode_distance(dist_code, reader)?;

            // Copy from back-reference
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
                // Repeat previous length 3-6 times
                let repeat = reader.read_bits(2)? as usize + 3;
                let prev = if i > 0 { all_lens[i-1] } else { 0 };
                for _ in 0..repeat { if i < total { all_lens[i] = prev; i += 1; } }
            }
            17 => {
                // Repeat 0 for 3-10 times
                let repeat = reader.read_bits(3)? as usize + 3;
                for _ in 0..repeat { if i < total { all_lens[i] = 0; i += 1; } }
            }
            18 => {
                // Repeat 0 for 11-138 times
                let repeat = reader.read_bits(7)? as usize + 11;
                for _ in 0..repeat { if i < total { all_lens[i] = 0; i += 1; } }
            }
            _ => return Err("invalid code length code"),
        }
    }

    // Build literal/length and distance Huffman tables
    let lit_table = build_huffman_table(&all_lens[..hlit], hlit);
    let dist_table = build_huffman_table(&all_lens[hlit..hlit+hdist], hdist);

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

    // Count occurrences of each code length
    for i in 0..count.min(MAX_HUFFMAN) {
        let l = lens[i] as usize;
        if l <= MAX_BITS { table.counts[l] += 1; }
    }
    table.counts[0] = 0;

    // Compute offsets
    let mut offset = 0u16;
    for i in 1..=MAX_BITS {
        table.offsets[i] = offset;
        offset += table.counts[i];
    }

    // Build sorted symbol table
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
    let mut index = 0u32;

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
        index += count;
    }

    Err("invalid huffman code")
}

// ─── DEFLATE fixed code decoding ───

fn decode_fixed_literal(reader: &mut BitReader) -> Result<u32, &'static str> {
    // Fixed Huffman: 0-143 = 7-8 bits, 144-255 = 9 bits, 256-279 = 7 bits, 280-287 = 8 bits
    let mut code = read_reversed_bits(reader, 7)?;

    if code <= 23 {
        // 256-279 (7-bit codes starting at 0b0000000)
        return Ok(code + 256);
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
    pos: usize,   // byte position
    bit: u8,      // bit position within current byte (0-7)
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
