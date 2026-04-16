#![allow(dead_code)]
// Bat_OS -- Minimal TrueType Font Rasterizer
// Parses TTF files, maps Unicode to glyphs, reads outlines, and rasterizes
// using scanline fill. No alloc, no std -- all fixed-size buffers.
//
// Place a TTF font file at fonts/font.ttf (e.g., DejaVu Sans, Liberation Sans,
// or Noto Sans). The font data is embedded via include_bytes!.

// ---------------------------------------------------------------------------
// Constants & limits
// ---------------------------------------------------------------------------

/// Maximum glyph bitmap dimension (pixels).
const MAX_GLYPH_SIZE: usize = 64;

/// Maximum number of points in a single glyph outline.
const MAX_POINTS: usize = 512;

/// Maximum number of contours in a single glyph.
const MAX_CONTOURS: usize = 64;

/// Maximum number of edges for scanline rasterization.
const MAX_EDGES: usize = 2048;

/// Maximum number of line segments after Bezier flattening.
const MAX_SEGMENTS: usize = 2048;

/// Maximum number of x-crossings per scanline.
const MAX_CROSSINGS: usize = 256;

/// Maximum number of compound glyph components.
const MAX_COMPONENTS: usize = 32;

// ---------------------------------------------------------------------------
// Big-endian reader helpers
// ---------------------------------------------------------------------------

#[inline]
fn u16be(data: &[u8], off: usize) -> u16 {
    if off + 2 > data.len() { return 0; }
    ((data[off] as u16) << 8) | (data[off + 1] as u16)
}

#[inline]
fn i16be(data: &[u8], off: usize) -> i16 {
    u16be(data, off) as i16
}

#[inline]
fn u32be(data: &[u8], off: usize) -> u32 {
    if off + 4 > data.len() { return 0; }
    ((data[off] as u32) << 24)
        | ((data[off + 1] as u32) << 16)
        | ((data[off + 2] as u32) << 8)
        | (data[off + 3] as u32)
}

#[inline]
fn tag_eq(data: &[u8], off: usize, tag: &[u8; 4]) -> bool {
    if off + 4 > data.len() { return false; }
    data[off] == tag[0] && data[off + 1] == tag[1]
        && data[off + 2] == tag[2] && data[off + 3] == tag[3]
}

// ---------------------------------------------------------------------------
// TrueTypeFont
// ---------------------------------------------------------------------------

pub struct TrueTypeFont {
    data: &'static [u8],
    // Table offsets
    cmap_offset: usize,
    glyf_offset: usize,
    loca_offset: usize,
    head_offset: usize,
    hhea_offset: usize,
    hmtx_offset: usize,
    maxp_offset: usize,
    // Font metrics
    units_per_em: u16,
    num_glyphs: u16,
    loca_format: u16, // 0=short, 1=long
    num_h_metrics: u16,
    ascent: i16,
    descent: i16,
}

/// A single point in a glyph outline.
#[derive(Clone, Copy)]
struct GlyphPoint {
    x: i32,
    y: i32,
    on_curve: bool,
}

/// A line segment for scanline rasterization.
#[derive(Clone, Copy)]
struct Edge {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

impl TrueTypeFont {
    /// Parse a TTF file from raw bytes. Returns None if invalid.
    pub fn parse(data: &'static [u8]) -> Option<Self> {
        if data.len() < 12 { return None; }

        // Check for TrueType magic: 0x00010000 or 'true'
        let sfversion = u32be(data, 0);
        if sfversion != 0x00010000 && sfversion != 0x74727565 {
            return None;
        }

        let num_tables = u16be(data, 4) as usize;
        if data.len() < 12 + num_tables * 16 { return None; }

        let mut cmap_offset = 0usize;
        let mut glyf_offset = 0usize;
        let mut loca_offset = 0usize;
        let mut head_offset = 0usize;
        let mut hhea_offset = 0usize;
        let mut hmtx_offset = 0usize;
        let mut maxp_offset = 0usize;

        // Walk the table directory
        for i in 0..num_tables {
            let rec = 12 + i * 16;
            let offset = u32be(data, rec + 8) as usize;

            if tag_eq(data, rec, b"cmap") { cmap_offset = offset; }
            else if tag_eq(data, rec, b"glyf") { glyf_offset = offset; }
            else if tag_eq(data, rec, b"loca") { loca_offset = offset; }
            else if tag_eq(data, rec, b"head") { head_offset = offset; }
            else if tag_eq(data, rec, b"hhea") { hhea_offset = offset; }
            else if tag_eq(data, rec, b"hmtx") { hmtx_offset = offset; }
            else if tag_eq(data, rec, b"maxp") { maxp_offset = offset; }
        }

        // Validate required tables
        if cmap_offset == 0 || glyf_offset == 0 || loca_offset == 0
            || head_offset == 0 || maxp_offset == 0
        {
            return None;
        }

        // head table: unitsPerEm at offset 18, indexToLocFormat at offset 50
        let units_per_em = u16be(data, head_offset + 18);
        let loca_format = u16be(data, head_offset + 50);

        // maxp table: numGlyphs at offset 4
        let num_glyphs = u16be(data, maxp_offset + 4);

        // hhea table: ascent(4), descent(6), numOfLongHorMetrics(34)
        let ascent = if hhea_offset != 0 { i16be(data, hhea_offset + 4) } else { units_per_em as i16 };
        let descent = if hhea_offset != 0 { i16be(data, hhea_offset + 6) } else { 0 };
        let num_h_metrics = if hhea_offset != 0 { u16be(data, hhea_offset + 34) } else { num_glyphs };

        Some(TrueTypeFont {
            data,
            cmap_offset,
            glyf_offset,
            loca_offset,
            head_offset,
            hhea_offset,
            hmtx_offset,
            maxp_offset,
            units_per_em,
            num_glyphs,
            loca_format,
            num_h_metrics,
            ascent,
            descent,
        })
    }

    // -----------------------------------------------------------------------
    // cmap: Unicode -> Glyph Index
    // -----------------------------------------------------------------------

    /// Look up the glyph index for a Unicode codepoint.
    pub fn glyph_index(&self, codepoint: u32) -> u16 {
        let data = self.data;
        let base = self.cmap_offset;
        if base == 0 { return 0; }

        let num_subtables = u16be(data, base + 2) as usize;

        // Find a suitable subtable: prefer (3,10) format 12, then (3,1) format 4,
        // then (0,3) format 4.
        let mut best_offset = 0usize;
        let mut best_format = 0u16;

        for i in 0..num_subtables {
            let rec = base + 4 + i * 8;
            let platform = u16be(data, rec);
            let encoding = u16be(data, rec + 2);
            let offset = u32be(data, rec + 4) as usize;
            let subtable = base + offset;
            let format = u16be(data, subtable);

            // (3,10) = Windows UCS-4 -> format 12
            if platform == 3 && encoding == 10 && format == 12 {
                best_offset = subtable;
                best_format = 12;
                break; // best possible
            }
            // (3,1) = Windows UCS-2 -> format 4
            if platform == 3 && encoding == 1 && format == 4 {
                if best_format < 4 {
                    best_offset = subtable;
                    best_format = 4;
                }
            }
            // (0,3) = Unicode BMP -> format 4
            if platform == 0 && encoding == 3 && format == 4 {
                if best_format < 4 {
                    best_offset = subtable;
                    best_format = 4;
                }
            }
        }

        if best_format == 12 {
            self.cmap_format12(best_offset, codepoint)
        } else if best_format == 4 {
            self.cmap_format4(best_offset, codepoint)
        } else {
            0
        }
    }

    fn cmap_format4(&self, subtable: usize, codepoint: u32) -> u16 {
        if codepoint > 0xFFFF { return 0; }
        let cp = codepoint as u16;
        let data = self.data;

        let seg_count = u16be(data, subtable + 6) / 2;
        let end_code_base = subtable + 14;
        let start_code_base = end_code_base + (seg_count as usize) * 2 + 2; // +2 for reservedPad
        let id_delta_base = start_code_base + (seg_count as usize) * 2;
        let id_range_base = id_delta_base + (seg_count as usize) * 2;

        for i in 0..(seg_count as usize) {
            let end_code = u16be(data, end_code_base + i * 2);
            if cp > end_code { continue; }

            let start_code = u16be(data, start_code_base + i * 2);
            if cp < start_code { return 0; }

            let id_delta = i16be(data, id_delta_base + i * 2);
            let id_range_offset = u16be(data, id_range_base + i * 2);

            if id_range_offset == 0 {
                return (cp as i32 + id_delta as i32) as u16;
            } else {
                let glyph_addr = id_range_base + i * 2
                    + id_range_offset as usize
                    + ((cp - start_code) as usize) * 2;
                let glyph_id = u16be(data, glyph_addr);
                if glyph_id != 0 {
                    return (glyph_id as i32 + id_delta as i32) as u16;
                }
                return 0;
            }
        }
        0
    }

    fn cmap_format12(&self, subtable: usize, codepoint: u32) -> u16 {
        let data = self.data;
        let n_groups = u32be(data, subtable + 12) as usize;
        let groups_base = subtable + 16;

        // Binary search
        let mut lo = 0usize;
        let mut hi = n_groups;
        while lo < hi {
            let mid = (lo + hi) / 2;
            let rec = groups_base + mid * 12;
            let start_code = u32be(data, rec);
            let end_code = u32be(data, rec + 4);
            let start_glyph = u32be(data, rec + 8);

            if codepoint < start_code {
                hi = mid;
            } else if codepoint > end_code {
                lo = mid + 1;
            } else {
                return (start_glyph + (codepoint - start_code)) as u16;
            }
        }
        0
    }

    // -----------------------------------------------------------------------
    // loca: Glyph offset in glyf table
    // -----------------------------------------------------------------------

    fn glyph_offset(&self, glyph_id: u16) -> usize {
        if glyph_id >= self.num_glyphs { return 0; }
        let data = self.data;
        let base = self.loca_offset;
        let id = glyph_id as usize;

        if self.loca_format == 0 {
            // Short format: stored as u16, actual offset = value * 2
            (u16be(data, base + id * 2) as usize) * 2
        } else {
            // Long format: stored as u32
            u32be(data, base + id * 4) as usize
        }
    }

    fn glyph_length(&self, glyph_id: u16) -> usize {
        if glyph_id + 1 >= self.num_glyphs { return 0; }
        let next = self.glyph_offset(glyph_id + 1);
        let cur = self.glyph_offset(glyph_id);
        if next > cur { next - cur } else { 0 }
    }

    // -----------------------------------------------------------------------
    // hmtx: Advance width
    // -----------------------------------------------------------------------

    fn advance_width(&self, glyph_id: u16) -> u16 {
        if self.hmtx_offset == 0 { return self.units_per_em; }
        let data = self.data;
        let base = self.hmtx_offset;

        if (glyph_id as u16) < self.num_h_metrics {
            u16be(data, base + (glyph_id as usize) * 4)
        } else {
            // Use last entry's advance width
            u16be(data, base + ((self.num_h_metrics as usize).saturating_sub(1)) * 4)
        }
    }

    // -----------------------------------------------------------------------
    // glyf: Read glyph outlines
    // -----------------------------------------------------------------------

    /// Read a simple glyph's outline points.
    /// Returns the number of points read.
    fn read_simple_glyph(
        &self,
        glyph_id: u16,
        points: &mut [GlyphPoint; MAX_POINTS],
        contour_ends: &mut [u16; MAX_CONTOURS],
    ) -> (usize, usize) {
        // (num_points, num_contours)
        let data = self.data;
        let offset = self.glyf_offset + self.glyph_offset(glyph_id);
        let glyph_len = self.glyph_length(glyph_id);
        if glyph_len == 0 { return (0, 0); }
        if offset + 10 > data.len() { return (0, 0); }

        let num_contours = i16be(data, offset);
        if num_contours <= 0 { return (0, 0); } // compound or empty
        let nc = num_contours as usize;
        if nc > MAX_CONTOURS { return (0, 0); }

        // endPtsOfContours starts at offset+10
        let mut max_pt = 0u16;
        for i in 0..nc {
            let ep = u16be(data, offset + 10 + i * 2);
            contour_ends[i] = ep;
            if ep > max_pt { max_pt = ep; }
        }
        let num_points = (max_pt + 1) as usize;
        if num_points > MAX_POINTS { return (0, 0); }

        // Skip instructions
        let instr_offset = offset + 10 + nc * 2;
        let instr_len = u16be(data, instr_offset) as usize;
        let flags_start = instr_offset + 2 + instr_len;

        // Read flags
        let mut flags = [0u8; MAX_POINTS];
        let mut fi = flags_start;
        let mut pi = 0;
        while pi < num_points && fi < data.len() {
            let flag = data[fi];
            fi += 1;
            flags[pi] = flag;
            pi += 1;

            // Repeat flag?
            if (flag & 0x08) != 0 && fi < data.len() {
                let repeat_count = data[fi] as usize;
                fi += 1;
                for _ in 0..repeat_count {
                    if pi >= num_points { break; }
                    flags[pi] = flag;
                    pi += 1;
                }
            }
        }

        // Read x-coordinates
        let mut x: i32 = 0;
        let mut xi = fi;
        for i in 0..num_points {
            let flag = flags[i];
            if (flag & 0x02) != 0 {
                // x is 1 byte
                if xi >= data.len() { break; }
                let dx = data[xi] as i32;
                xi += 1;
                if (flag & 0x10) != 0 { x += dx; } else { x -= dx; }
            } else if (flag & 0x10) == 0 {
                // x is 2 bytes (signed)
                if xi + 2 > data.len() { break; }
                let dx = i16be(data, xi) as i32;
                xi += 2;
                x += dx;
            }
            // else: x is same as previous (flag & 0x10 set, not short)
            points[i].x = x;
            points[i].on_curve = (flag & 0x01) != 0;
        }

        // Read y-coordinates
        let mut y: i32 = 0;
        let mut yi = xi;
        for i in 0..num_points {
            let flag = flags[i];
            if (flag & 0x04) != 0 {
                // y is 1 byte
                if yi >= data.len() { break; }
                let dy = data[yi] as i32;
                yi += 1;
                if (flag & 0x20) != 0 { y += dy; } else { y -= dy; }
            } else if (flag & 0x20) == 0 {
                // y is 2 bytes (signed)
                if yi + 2 > data.len() { break; }
                let dy = i16be(data, yi) as i32;
                yi += 2;
                y += dy;
            }
            points[i].y = y;
        }

        (num_points, nc)
    }

    /// Read a compound glyph by composing its components.
    fn read_compound_glyph(
        &self,
        glyph_id: u16,
        points: &mut [GlyphPoint; MAX_POINTS],
        contour_ends: &mut [u16; MAX_CONTOURS],
    ) -> (usize, usize) {
        let data = self.data;
        let offset = self.glyf_offset + self.glyph_offset(glyph_id);
        let glyph_len = self.glyph_length(glyph_id);
        if glyph_len == 0 { return (0, 0); }
        if offset + 10 > data.len() { return (0, 0); }

        let num_contours = i16be(data, offset);
        if num_contours >= 0 { return (0, 0); } // not compound

        let mut total_points = 0usize;
        let mut total_contours = 0usize;
        let mut pos = offset + 10;

        for _comp in 0..MAX_COMPONENTS {
            if pos + 4 > data.len() { break; }
            let flags = u16be(data, pos);
            let component_glyph = u16be(data, pos + 2);
            pos += 4;

            // Read translation offsets
            let dx: i32;
            let dy: i32;
            if (flags & 0x0001) != 0 {
                // ARG_1_AND_2_ARE_WORDS
                if pos + 4 > data.len() { break; }
                if (flags & 0x0002) != 0 {
                    // ARGS_ARE_XY_VALUES
                    dx = i16be(data, pos) as i32;
                    dy = i16be(data, pos + 2) as i32;
                } else {
                    dx = 0;
                    dy = 0;
                }
                pos += 4;
            } else {
                if pos + 2 > data.len() { break; }
                if (flags & 0x0002) != 0 {
                    dx = (data[pos] as i8) as i32;
                    dy = (data[pos + 1] as i8) as i32;
                } else {
                    dx = 0;
                    dy = 0;
                }
                pos += 2;
            }

            // Read scale/transform (simplified: just skip)
            let mut scale_x: f32 = 1.0;
            let mut scale_y: f32 = 1.0;
            if (flags & 0x0008) != 0 {
                // WE_HAVE_A_SCALE
                if pos + 2 > data.len() { break; }
                let s = i16be(data, pos) as f32 / 16384.0;
                scale_x = s;
                scale_y = s;
                pos += 2;
            } else if (flags & 0x0040) != 0 {
                // WE_HAVE_AN_X_AND_Y_SCALE
                if pos + 4 > data.len() { break; }
                scale_x = i16be(data, pos) as f32 / 16384.0;
                scale_y = i16be(data, pos + 2) as f32 / 16384.0;
                pos += 4;
            } else if (flags & 0x0080) != 0 {
                // WE_HAVE_A_TWO_BY_TWO
                if pos + 8 > data.len() { break; }
                scale_x = i16be(data, pos) as f32 / 16384.0;
                scale_y = i16be(data, pos + 6) as f32 / 16384.0;
                pos += 8;
            }

            // Recursively read the component glyph
            let mut comp_points = [GlyphPoint { x: 0, y: 0, on_curve: true }; MAX_POINTS];
            let mut comp_contours = [0u16; MAX_CONTOURS];
            let (cp, cc) = self.read_glyph_outline(component_glyph, &mut comp_points, &mut comp_contours);

            // Merge into output, applying transform
            for i in 0..cc {
                if total_contours + i >= MAX_CONTOURS { break; }
                contour_ends[total_contours + i] = comp_contours[i] + total_points as u16;
            }
            for i in 0..cp {
                if total_points + i >= MAX_POINTS { break; }
                points[total_points + i] = GlyphPoint {
                    x: (comp_points[i].x as f32 * scale_x) as i32 + dx,
                    y: (comp_points[i].y as f32 * scale_y) as i32 + dy,
                    on_curve: comp_points[i].on_curve,
                };
            }
            total_points += cp;
            total_contours += cc;

            // MORE_COMPONENTS flag
            if (flags & 0x0020) == 0 { break; }
        }

        (total_points, total_contours)
    }

    /// Read glyph outline (simple or compound).
    fn read_glyph_outline(
        &self,
        glyph_id: u16,
        points: &mut [GlyphPoint; MAX_POINTS],
        contour_ends: &mut [u16; MAX_CONTOURS],
    ) -> (usize, usize) {
        let data = self.data;
        let offset = self.glyf_offset + self.glyph_offset(glyph_id);
        let glyph_len = self.glyph_length(glyph_id);
        if glyph_len == 0 || offset + 10 > data.len() { return (0, 0); }

        let num_contours = i16be(data, offset);
        if num_contours > 0 {
            self.read_simple_glyph(glyph_id, points, contour_ends)
        } else if num_contours == -1 {
            self.read_compound_glyph(glyph_id, points, contour_ends)
        } else {
            (0, 0) // empty glyph (e.g., space)
        }
    }

    // -----------------------------------------------------------------------
    // Outline -> Edge list (with Bezier flattening)
    // -----------------------------------------------------------------------

    /// Flatten a quadratic Bezier (p0, control, p1) into line segments.
    /// Appends to `edges` starting at `edge_count`. Returns new edge_count.
    fn flatten_bezier(
        p0x: f32, p0y: f32,
        cx: f32, cy: f32,
        p1x: f32, p1y: f32,
        edges: &mut [Edge; MAX_EDGES],
        mut edge_count: usize,
        depth: u32,
    ) -> usize {
        // Flatness test: if the control point is close to the midpoint of p0-p1,
        // the curve is flat enough to approximate with a line.
        let mx = (p0x + p1x) * 0.5;
        let my = (p0y + p1y) * 0.5;
        let dx = cx - mx;
        let dy = cy - my;
        let flatness = dx * dx + dy * dy;

        if flatness < 0.25 || depth > 6 {
            // Flat enough: emit line segment
            if edge_count < MAX_EDGES {
                edges[edge_count] = Edge { x0: p0x, y0: p0y, x1: p1x, y1: p1y };
                edge_count += 1;
            }
        } else {
            // Subdivide at t=0.5
            let q0x = (p0x + cx) * 0.5;
            let q0y = (p0y + cy) * 0.5;
            let q1x = (cx + p1x) * 0.5;
            let q1y = (cy + p1y) * 0.5;
            let qmx = (q0x + q1x) * 0.5;
            let qmy = (q0y + q1y) * 0.5;

            edge_count = Self::flatten_bezier(p0x, p0y, q0x, q0y, qmx, qmy, edges, edge_count, depth + 1);
            edge_count = Self::flatten_bezier(qmx, qmy, q1x, q1y, p1x, p1y, edges, edge_count, depth + 1);
        }
        edge_count
    }

    /// Convert glyph outline points to edge list.
    /// Scale factor maps font units to pixels.
    fn outline_to_edges(
        points: &[GlyphPoint; MAX_POINTS],
        contour_ends: &[u16; MAX_CONTOURS],
        num_points: usize,
        num_contours: usize,
        scale: f32,
        offset_x: f32,
        offset_y: f32,
        edges: &mut [Edge; MAX_EDGES],
    ) -> usize {
        let mut edge_count = 0usize;
        let mut contour_start = 0usize;

        for c in 0..num_contours {
            let contour_end = contour_ends[c] as usize;
            if contour_end >= num_points { break; }
            let n = contour_end - contour_start + 1;
            if n < 2 { contour_start = contour_end + 1; continue; }

            // Process contour -- handle implicit on-curve points between off-curve points
            let mut i = 0usize;
            while i < n {
                let cur_idx = contour_start + i;
                let next_idx = contour_start + ((i + 1) % n);

                let p0 = points[cur_idx];
                let p1 = points[next_idx];

                let sx0 = p0.x as f32 * scale + offset_x;
                let sy0 = offset_y - p0.y as f32 * scale; // Y is flipped

                if p0.on_curve && p1.on_curve {
                    // Line segment
                    let sx1 = p1.x as f32 * scale + offset_x;
                    let sy1 = offset_y - p1.y as f32 * scale;
                    if edge_count < MAX_EDGES {
                        edges[edge_count] = Edge { x0: sx0, y0: sy0, x1: sx1, y1: sy1 };
                        edge_count += 1;
                    }
                    i += 1;
                } else if p0.on_curve && !p1.on_curve {
                    // Start of Bezier: on -> off -> ...
                    let cx = p1.x as f32 * scale + offset_x;
                    let cy = offset_y - p1.y as f32 * scale;

                    let next2_idx = contour_start + ((i + 2) % n);
                    let p2 = points[next2_idx];

                    if p2.on_curve {
                        // on -> off -> on: standard quadratic Bezier
                        let sx2 = p2.x as f32 * scale + offset_x;
                        let sy2 = offset_y - p2.y as f32 * scale;
                        edge_count = Self::flatten_bezier(sx0, sy0, cx, cy, sx2, sy2, edges, edge_count, 0);
                        i += 2;
                    } else {
                        // on -> off -> off: implicit on-curve at midpoint
                        let sx2 = p2.x as f32 * scale + offset_x;
                        let sy2 = offset_y - p2.y as f32 * scale;
                        let mid_x = (cx + sx2) * 0.5;
                        let mid_y = (cy + sy2) * 0.5;
                        edge_count = Self::flatten_bezier(sx0, sy0, cx, cy, mid_x, mid_y, edges, edge_count, 0);
                        i += 1; // advance to p1 (off-curve), next iteration handles from implicit point
                    }
                } else if !p0.on_curve && p1.on_curve {
                    // off -> on: need previous point context
                    // This case is handled by the previous iteration's implicit mid-point
                    // Since we process pairs starting from on-curve, this handles the
                    // tail of an implicit-midpoint Bezier.
                    let prev_idx = contour_start + ((i + n - 1) % n);
                    let pp = points[prev_idx];
                    let mid_x = (pp.x as f32 * scale + offset_x + sx0) * 0.5;
                    let mid_y = (offset_y - pp.y as f32 * scale + sy0) * 0.5;

                    let sx1 = p1.x as f32 * scale + offset_x;
                    let sy1 = offset_y - p1.y as f32 * scale;
                    edge_count = Self::flatten_bezier(mid_x, mid_y, sx0, sy0, sx1, sy1, edges, edge_count, 0);
                    i += 1;
                } else {
                    // off -> off: implicit on-curve midpoint between them
                    let sx1 = p1.x as f32 * scale + offset_x;
                    let sy1 = offset_y - p1.y as f32 * scale;
                    let _mid0_x = (sx0 + offset_x) * 0.5; // midpoint before p0
                    let _mid0_y = (sy0 + offset_y) * 0.5;
                    let mid1_x = (sx0 + sx1) * 0.5;
                    let mid1_y = (sy0 + sy1) * 0.5;
                    edge_count = Self::flatten_bezier(mid1_x, mid1_y, sx0, sy0, mid1_x, mid1_y, edges, edge_count, 0);
                    i += 1;
                }
            }

            contour_start = contour_end + 1;
        }

        edge_count
    }

    // -----------------------------------------------------------------------
    // Scanline rasterizer
    // -----------------------------------------------------------------------

    /// Rasterize edges into a bitmap using scanline fill (even-odd rule).
    /// bitmap is row-major, one byte per pixel (0 = empty, 255 = filled).
    fn scanline_fill(
        edges: &[Edge; MAX_EDGES],
        edge_count: usize,
        bitmap: &mut [u8],
        bmp_w: usize,
        bmp_h: usize,
    ) {
        let mut crossings = [0.0f32; MAX_CROSSINGS];

        for y in 0..bmp_h {
            let scanline_y = y as f32 + 0.5; // sample at pixel center
            let mut num_crossings = 0usize;

            // Find all edge crossings with this scanline
            for ei in 0..edge_count {
                let e = &edges[ei];
                let (y_min, y_max) = if e.y0 < e.y1 { (e.y0, e.y1) } else { (e.y1, e.y0) };

                // Edge crosses this scanline?
                if scanline_y < y_min || scanline_y >= y_max { continue; }

                // Compute x at the crossing
                let dy = e.y1 - e.y0;
                if dy.abs() < 0.0001 { continue; } // horizontal edge
                let t = (scanline_y - e.y0) / dy;
                let x = e.x0 + t * (e.x1 - e.x0);

                if num_crossings < MAX_CROSSINGS {
                    crossings[num_crossings] = x;
                    num_crossings += 1;
                }
            }

            // Sort crossings (insertion sort -- small N)
            for i in 1..num_crossings {
                let val = crossings[i];
                let mut j = i;
                while j > 0 && crossings[j - 1] > val {
                    crossings[j] = crossings[j - 1];
                    j -= 1;
                }
                crossings[j] = val;
            }

            // Fill between pairs of crossings (even-odd rule)
            let mut ci = 0;
            while ci + 1 < num_crossings {
                let x_start = crossings[ci];
                let x_end = crossings[ci + 1];

                let px_start = (x_start as i32).max(0) as usize;
                let px_end = ((x_end as i32) + 1).min(bmp_w as i32) as usize;

                for px in px_start..px_end {
                    if px < bmp_w {
                        let idx = y * bmp_w + px;
                        if idx < bitmap.len() {
                            // Basic coverage: fully inside = 255
                            let frac_x = px as f32;
                            if frac_x >= x_start && frac_x + 1.0 <= x_end {
                                bitmap[idx] = 255;
                            } else {
                                // Partial coverage for anti-aliasing at edges
                                let coverage = if frac_x < x_start {
                                    ((frac_x + 1.0 - x_start) * 255.0) as u8
                                } else if frac_x + 1.0 > x_end {
                                    ((x_end - frac_x) * 255.0) as u8
                                } else {
                                    255
                                };
                                // Blend with existing value (max)
                                if coverage > bitmap[idx] {
                                    bitmap[idx] = coverage;
                                }
                            }
                        }
                    }
                }

                ci += 2;
            }
        }
    }

    // -----------------------------------------------------------------------
    // Public API
    // -----------------------------------------------------------------------

    /// Render a single character to a bitmap buffer.
    /// `bitmap` must be at least MAX_GLYPH_SIZE * MAX_GLYPH_SIZE bytes.
    /// Returns (width, height, advance_width_px) of the rendered glyph.
    pub fn render_char(&self, ch: char, size_px: u16, bitmap: &mut [u8]) -> (u16, u16, u16) {
        let codepoint = ch as u32;
        let glyph_id = self.glyph_index(codepoint);

        // Scale factor: font units -> pixels
        let scale = size_px as f32 / self.units_per_em as f32;

        // Advance width in pixels
        let advance_fu = self.advance_width(glyph_id);
        let advance_px = (advance_fu as f32 * scale + 0.5) as u16;

        // Compute bitmap dimensions
        let bmp_w = (advance_px as usize).min(MAX_GLYPH_SIZE).max(1);
        let ascent_px = (self.ascent as f32 * scale) as usize;
        let descent_px = (-(self.descent as f32) * scale) as usize;
        let bmp_h = (ascent_px + descent_px).min(MAX_GLYPH_SIZE).max(1);

        // Clear bitmap
        let total = bmp_w * bmp_h;
        for i in 0..total.min(bitmap.len()) {
            bitmap[i] = 0;
        }

        // Read glyph outline
        let mut points = [GlyphPoint { x: 0, y: 0, on_curve: true }; MAX_POINTS];
        let mut contour_ends = [0u16; MAX_CONTOURS];
        let (num_points, num_contours) = self.read_glyph_outline(glyph_id, &mut points, &mut contour_ends);

        if num_points == 0 || num_contours == 0 {
            // No outline (e.g., space character)
            return (bmp_w as u16, bmp_h as u16, advance_px);
        }

        // Convert outline to edges
        let mut edges = [Edge { x0: 0.0, y0: 0.0, x1: 0.0, y1: 0.0 }; MAX_EDGES];
        let offset_y = ascent_px as f32;
        let edge_count = Self::outline_to_edges(
            &points, &contour_ends, num_points, num_contours,
            scale, 0.0, offset_y, &mut edges,
        );

        // Rasterize
        Self::scanline_fill(&edges, edge_count, bitmap, bmp_w, bmp_h);

        (bmp_w as u16, bmp_h as u16, advance_px)
    }

    /// Render a string to the framebuffer at (x, y).
    /// `fb` is the framebuffer as a slice of u32 (ARGB).
    /// `fb_w` is the framebuffer width in pixels.
    pub fn draw_string(
        &self,
        fb: &mut [u8],
        fb_w: u32,
        x: u32,
        y: u32,
        text: &str,
        size_px: u16,
        color: u32,
    ) {
        let mut cursor_x = x;
        let mut bitmap = [0u8; MAX_GLYPH_SIZE * MAX_GLYPH_SIZE];

        let r = ((color >> 16) & 0xFF) as u32;
        let g = ((color >> 8) & 0xFF) as u32;
        let b = (color & 0xFF) as u32;
        let a = ((color >> 24) & 0xFF) as u32;

        for ch in text.chars() {
            // Handle newlines
            if ch == '\n' {
                // Not handled here — caller should split lines
                continue;
            }

            let (gw, gh, advance) = self.render_char(ch, size_px, &mut bitmap);

            // Blit bitmap to framebuffer with alpha blending
            for row in 0..gh as u32 {
                let screen_y = y + row;
                if screen_y >= (fb.len() as u32 / fb_w) { break; }

                for col in 0..gw as u32 {
                    let screen_x = cursor_x + col;
                    if screen_x >= fb_w { break; }

                    let coverage = bitmap[(row * gw as u32 + col) as usize] as u32;
                    if coverage == 0 { continue; }

                    let fb_idx = ((screen_y * fb_w + screen_x) * 4) as usize;
                    if fb_idx + 3 >= fb.len() { continue; }

                    if coverage >= 250 {
                        // Fully opaque -- BGRA format
                        fb[fb_idx] = b as u8;     // B
                        fb[fb_idx+1] = g as u8;   // G
                        fb[fb_idx+2] = r as u8;   // R
                        fb[fb_idx+3] = 0xFF;      // A
                    } else {
                        // Alpha blend with existing pixel (BGRA)
                        let bg_b = fb[fb_idx] as u32;
                        let bg_g = fb[fb_idx+1] as u32;
                        let bg_r = fb[fb_idx+2] as u32;

                        let alpha = (coverage * a) / 255;
                        let inv_alpha = 255 - alpha;
                        fb[fb_idx]   = ((b * alpha + bg_b * inv_alpha) / 255) as u8;
                        fb[fb_idx+1] = ((g * alpha + bg_g * inv_alpha) / 255) as u8;
                        fb[fb_idx+2] = ((r * alpha + bg_r * inv_alpha) / 255) as u8;
                        fb[fb_idx+3] = 0xFF;
                    }
                }
            }

            cursor_x += advance as u32;
        }
    }
}

// ---------------------------------------------------------------------------
// Convenience function for the browser/paint module
// ---------------------------------------------------------------------------

/// Draw text using TrueType font rendering.
/// Falls back to bitmap font if no TTF font is available.
/// `fb` is the raw framebuffer pointer, `fb_w` is framebuffer width.
pub fn draw_truetype(
    fb: &mut [u8],
    fb_w: u32,
    x: u32,
    y: u32,
    text: &str,
    size: u16,
    color: u32,
) {
    // Attempt to use embedded TTF font
    if let Some(font) = get_font() {
        font.draw_string(fb, fb_w, x, y, text, size, color);
    }
    // If no font available, caller should fall back to bitmap font
}

/// Cached font singleton (parsed once, reused).
/// Uses raw pointer access (addr_of/addr_of_mut) for Rust 2024 compatibility.
static mut FONT_CACHE: Option<TrueTypeFont> = None;
static mut FONT_INIT: bool = false;

/// Embedded Verdana font (186KB)
static EMBEDDED_FONT: &[u8] = include_bytes!("../../fonts/font.ttf");

/// Get the font, parsing it on first call.
fn get_font() -> Option<&'static TrueTypeFont> {
    unsafe {
        if !core::ptr::read_volatile(core::ptr::addr_of!(FONT_INIT)) {
            core::ptr::write_volatile(core::ptr::addr_of_mut!(FONT_INIT), true);
            let parsed = TrueTypeFont::parse(EMBEDDED_FONT);
            let has_font = parsed.is_some();
            core::ptr::write(core::ptr::addr_of_mut!(FONT_CACHE), parsed);
            if has_font {
                crate::drivers::uart::puts("[font] TrueType font loaded (Verdana, ");
                crate::kernel::mm::print_num(EMBEDDED_FONT.len());
                crate::drivers::uart::puts(" bytes)\n");
            }
        }
        let cache_ptr = core::ptr::addr_of!(FONT_CACHE);
        (*cache_ptr).as_ref()
    }
}

/// Initialize the TrueType font system with externally-provided font data.
/// Call this during kernel init if you have font data loaded from disk or
/// embedded via another mechanism.
pub fn init_with_data(data: &'static [u8]) {
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(FONT_INIT), true);
        core::ptr::write(core::ptr::addr_of_mut!(FONT_CACHE), TrueTypeFont::parse(data));
    }
}

/// Check if a TrueType font is available.
pub fn is_available() -> bool {
    get_font().is_some()
}

/// Draw anti-aliased text directly to the GPU framebuffer.
/// Uses the embedded TrueType font with alpha blending.
/// Returns the width of the rendered text in pixels.
pub fn draw_text_fb(
    fb: *mut u32,
    screen_w: u32,
    x: i32,
    y: i32,
    text: &[u8],
    size_px: u16,
    color: u32,
    clip_left: i32,
    clip_right: i32,
    clip_top: i32,
    clip_bottom: i32,
) -> i32 {
    let font = match get_font() {
        Some(f) => f,
        None => return 0,
    };

    let cr = ((color >> 16) & 0xFF) as u32;
    let cg = ((color >> 8) & 0xFF) as u32;
    let cb = (color & 0xFF) as u32;

    let mut cursor_x = x;
    let mut bitmap = [0u8; MAX_GLYPH_SIZE * MAX_GLYPH_SIZE];

    for &ch in text {
        if ch < 0x20 || ch > 0x7E { continue; }

        let (gw, gh, advance) = font.render_char(ch as char, size_px, &mut bitmap);

        // Blit glyph with alpha blending and clipping
        for row in 0..gh as i32 {
            let sy = y + row;
            if sy < clip_top || sy >= clip_bottom { continue; }

            for col in 0..gw as i32 {
                let sx = cursor_x + col;
                if sx < clip_left || sx >= clip_right { continue; }

                let coverage = bitmap[(row as usize) * (gw as usize) + (col as usize)] as u32;
                if coverage == 0 { continue; }

                let fb_idx = (sy as u32 * screen_w + sx as u32) as usize;
                unsafe {
                    let dst = core::ptr::read_volatile(fb.add(fb_idx));
                    let dr = (dst >> 16) & 0xFF;
                    let dg = (dst >> 8) & 0xFF;
                    let db = dst & 0xFF;

                    let r = dr + ((cr - dr) * coverage) / 255;
                    let g = dg + ((cg - dg) * coverage) / 255;
                    let b = db + ((cb - db) * coverage) / 255;

                    core::ptr::write_volatile(
                        fb.add(fb_idx),
                        0xFF000000 | (r << 16) | (g << 8) | b,
                    );
                }
            }
        }
        cursor_x += advance as i32;
    }

    cursor_x - x // return total width
}

/// Measure the width of text at a given pixel size without rendering.
pub fn text_width(text: &[u8], size_px: u16) -> i32 {
    let font = match get_font() {
        Some(f) => f,
        None => return text.len() as i32 * 8, // fallback: 8px per char
    };

    let mut width = 0i32;
    for &ch in text {
        if ch < 0x20 || ch > 0x7E { continue; }
        let mut dummy = [0u8; 4]; // tiny buffer, we only need advance
        let (_, _, advance) = font.render_char(ch as char, size_px, &mut dummy);
        width += advance as i32;
    }
    width
}
