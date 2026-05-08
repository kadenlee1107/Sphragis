#![allow(dead_code)]
// Bat_OS — Apple Device Tree (ADT) parser
//
// Apple's firmware (iBoot, then m1n1 when chainloading) hands the OS a
// binary device-tree blob describing every hardware register map on the
// SoC. This is the ONLY authoritative source of MMIO addresses per
// chip/machine — hardcoding addresses (as src/drivers/apple/soc.rs
// currently does for M1) produces wrong results on M4, M4 Pro, M4 Max.
//
// Format (NOT the same as FDT — Apple invented their own):
//
//   AdtNodeHdr { property_count: u32, child_count: u32 }   // 8 bytes
//     AdtPropHdr { name: [u8; 32], size: u32 }             // 36 bytes
//       value: [u8; size]                                   // padded to 4B
//     ... property_count times
//     ... child_count sub-nodes (recursive, same layout)
//
// Nodes don't carry a direct "name" field — the node name lives in a
// property called "name" that is conventionally the first property.
//
// `reg` property layout depends on the parent's `#address-cells` and
// `#size-cells` (like FDT). Buses (e.g. /arm-io) carry a `ranges`
// property that translates child addresses to absolute physical
// addresses — get_reg() walks this chain.
//
// References (used as protocol references only; clean-room impl):
//   m1n1/rust/src/adt.rs                (MIT)
//   AsahiLinux/docs/hw/soc/memmap.md    (CC-BY-SA)
//
// Security: every pointer walk is bounds-checked against `blob.len()`.
// Any malformed node or property returns `AdtError::BadOffset` instead
// of reading OOB — critical because iBoot's ADT lands in memory the
// attacker could theoretically scribble on before we parse it.

// ─── Error type ─────────────────────────────────────────────────────

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AdtError {
    /// Named property / node was not found under this parent.
    NotFound,
    /// Offset is out of blob bounds, mis-aligned, or references a
    /// malformed node/property header.
    BadOffset,
    /// Path contained segments that don't exist in the tree.
    BadPath,
    /// #address-cells / #size-cells was outside the supported range.
    BadCells,
    /// Property value didn't match the requested typed accessor
    /// (e.g. caller wanted a u32 but the property is 7 bytes).
    BadLength,
    /// Property declared a size that would spill past the blob end.
    OutOfBounds,
    /// Property value wasn't valid UTF-8 when asked for a string.
    NotString,
}

// ─── On-disk structures (reinterpreted from the blob) ───────────────

const ADT_ALIGN: usize = 4;
const NODE_HDR_SIZE: usize = 8;
const PROP_HDR_SIZE: usize = 32 + 4; // name + size

/// Round `n` up to the next multiple of `ADT_ALIGN`. Used for
/// property-value stride.
#[inline]
const fn align_up(n: usize) -> usize {
    (n + (ADT_ALIGN - 1)) & !(ADT_ALIGN - 1)
}

// ─── Top-level handle ────────────────────────────────────────────────

/// Borrowed view over an ADT blob.
///
/// Construction is `O(1)` — we don't walk the tree upfront; every lookup
/// is lazy. `blob.len()` bounds every subsequent offset.
#[derive(Copy, Clone)]
pub struct Adt<'a> {
    blob: &'a [u8],
}

impl<'a> Adt<'a> {
    /// Wrap a raw ADT blob. Does minimal validation — the root node's
    /// header is checked, but the whole tree is not eagerly walked.
    pub fn new(blob: &'a [u8]) -> Result<Self, AdtError> {
        if blob.len() < NODE_HDR_SIZE {
            return Err(AdtError::OutOfBounds);
        }
        let this = Adt { blob };
        // Validate root header
        this.read_node_hdr(0)?;
        Ok(this)
    }

    /// Build an Adt view from a raw pointer + size (what m1n1 hands us
    /// via `M1n1BootArgs.devtree_addr` / `devtree_size`).
    ///
    /// # Safety
    /// Caller guarantees `ptr..ptr+size` is a valid readable region
    /// that lives at least as long as `'a`.
    pub unsafe fn from_raw(ptr: *const u8, size: usize) -> Result<Self, AdtError> {
        if ptr.is_null() || size < NODE_HDR_SIZE {
            return Err(AdtError::OutOfBounds);
        }
        let blob = unsafe { core::slice::from_raw_parts(ptr, size) };
        Self::new(blob)
    }

    /// Root node (always at offset 0).
    pub fn root(self) -> Result<Node<'a>, AdtError> {
        Ok(Node { adt: self, offset: 0 })
    }

    /// Convenience: look up a node by '/'-delimited path starting from root.
    /// Returns `BadPath` for any missing segment.
    pub fn find(self, path: &str) -> Result<Node<'a>, AdtError> {
        let mut node = self.root()?;
        for seg in path.split('/').filter(|s| !s.is_empty()) {
            node = node.subnode(seg).map_err(|e| match e {
                AdtError::NotFound => AdtError::BadPath,
                other => other,
            })?;
        }
        Ok(node)
    }

    // ─── internal helpers (bounds-checked blob reads) ───

    #[inline]
    fn read_u32(self, off: usize) -> Result<u32, AdtError> {
        let end = off.checked_add(4).ok_or(AdtError::BadOffset)?;
        if end > self.blob.len() {
            return Err(AdtError::BadOffset);
        }
        let bytes: [u8; 4] = self.blob[off..end].try_into()
            .expect("adt: bounds-checked 4-byte slice → [u8; 4] is infallible");
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_node_hdr(self, off: usize) -> Result<AdtNodeHdr, AdtError> {
        if off % ADT_ALIGN != 0 {
            return Err(AdtError::BadOffset);
        }
        let pc = self.read_u32(off)?;
        let cc = self.read_u32(off + 4)?;
        // Sanity caps — a genuine ADT never has more than a few thousand
        // of either. A compromised ADT could declare billions to trick
        // us into integer-overflow math.
        if pc > 4096 || cc > 4096 || pc == 0 {
            return Err(AdtError::BadOffset);
        }
        Ok(AdtNodeHdr { property_count: pc, child_count: cc })
    }

    /// Read a property header at `off` and return (hdr, value_offset,
    /// next_offset). `next_offset` is the start of the next property or
    /// the start of the first child if this was the last property.
    fn read_prop_hdr(self, off: usize) -> Result<(PropHdrRef<'a>, usize, usize), AdtError> {
        if off % ADT_ALIGN != 0 {
            return Err(AdtError::BadOffset);
        }
        let name_end = off.checked_add(32).ok_or(AdtError::BadOffset)?;
        if name_end > self.blob.len() {
            return Err(AdtError::BadOffset);
        }
        let size = self.read_u32(name_end)?;
        // The top two bits of `size` are sometimes used by iBoot as
        // flags (e.g. "placeholder"). Mask them off — the real size is
        // in the low 30 bits.
        let size_real = (size & 0x3FFF_FFFF) as usize;
        let value_off = name_end + 4;
        let next = value_off.checked_add(align_up(size_real))
            .ok_or(AdtError::BadOffset)?;
        if next > self.blob.len() {
            return Err(AdtError::OutOfBounds);
        }
        let name = &self.blob[off..name_end];
        Ok((
            PropHdrRef { name_bytes: name, size: size_real },
            value_off,
            next,
        ))
    }

    fn prop_value(self, value_off: usize, size: usize) -> Result<&'a [u8], AdtError> {
        let end = value_off.checked_add(size).ok_or(AdtError::BadOffset)?;
        if end > self.blob.len() {
            return Err(AdtError::OutOfBounds);
        }
        Ok(&self.blob[value_off..end])
    }
}

// ─── Node header + property header structs ─────────────────────────

#[derive(Copy, Clone)]
struct AdtNodeHdr {
    property_count: u32,
    child_count: u32,
}

#[derive(Copy, Clone)]
struct PropHdrRef<'a> {
    name_bytes: &'a [u8], // 32 bytes, nul-terminated
    size: usize,
}

impl<'a> PropHdrRef<'a> {
    fn name(&self) -> &'a str {
        let nul = self.name_bytes.iter().position(|&b| b == 0).unwrap_or(32);
        core::str::from_utf8(&self.name_bytes[..nul]).unwrap_or("")
    }
}

// ─── Node API ───────────────────────────────────────────────────────

#[derive(Copy, Clone)]
pub struct Node<'a> {
    adt: Adt<'a>,
    /// Byte offset of this node's header within the blob.
    offset: usize,
}

impl<'a> Node<'a> {
    /// Read (or re-read) this node's header. Always bounds-checked.
    fn hdr(self) -> Result<AdtNodeHdr, AdtError> {
        self.adt.read_node_hdr(self.offset)
    }

    /// Byte offset of this node's FIRST property. Always immediately
    /// after the 8-byte node header.
    fn first_prop_offset(self) -> usize {
        self.offset + NODE_HDR_SIZE
    }

    /// Walk past all properties and return the offset of the first
    /// child node. If `child_count == 0` this still returns a valid
    /// offset (it would point at whatever comes after this node), so
    /// callers must check `child_count` first.
    fn children_start_offset(self) -> Result<usize, AdtError> {
        let hdr = self.hdr()?;
        let mut off = self.first_prop_offset();
        for _ in 0..hdr.property_count {
            let (_ph, _voff, next) = self.adt.read_prop_hdr(off)?;
            off = next;
        }
        Ok(off)
    }

    /// Size of this node (header + all properties + all descendant
    /// subtrees), in bytes. Used to skip to a sibling.
    ///
    /// Bounded: caps recursion at 16 levels (real ADTs peak ~10-12)
    /// and total node visits at 4096 across the whole call. Prevents
    /// adversarial or locally-corrupt ADTs from locking us up at the
    /// slow pre-cpufreq M4 boot clock — observed symptom was
    /// `/arm-io/dart-disp0` lookup hanging until iBoot watchdog reset
    /// the Mac.
    fn total_size(self) -> Result<usize, AdtError> {
        let mut budget: u32 = 4096;
        self.total_size_bounded(16, &mut budget)
    }

    fn total_size_bounded(
        self,
        depth_remaining: u32,
        budget: &mut u32,
    ) -> Result<usize, AdtError> {
        if depth_remaining == 0 || *budget == 0 {
            return Err(AdtError::BadOffset);
        }
        *budget -= 1;
        let mut off = self.children_start_offset()?;
        let hdr = self.hdr()?;
        for _ in 0..hdr.child_count {
            let child = Node { adt: self.adt, offset: off };
            off += child.total_size_bounded(depth_remaining - 1, budget)?;
        }
        Ok(off - self.offset)
    }

    /// Iterate this node's direct properties.
    pub fn properties(self) -> PropIter<'a> {
        let count = self.hdr().map(|h| h.property_count).unwrap_or(0);
        PropIter {
            adt: self.adt,
            offset: self.first_prop_offset(),
            remaining: count,
        }
    }

    /// Iterate this node's direct children.
    pub fn children(self) -> ChildIter<'a> {
        let (count, start) = match self.hdr() {
            Ok(h) => (h.child_count, self.children_start_offset().unwrap_or(0)),
            Err(_) => (0, 0),
        };
        ChildIter {
            adt: self.adt,
            offset: start,
            remaining: count,
        }
    }

    /// Find a direct property by name.
    pub fn prop(self, name: &str) -> Result<Property<'a>, AdtError> {
        for p in self.properties() {
            if p.name() == name {
                return Ok(p);
            }
        }
        Err(AdtError::NotFound)
    }

    /// The node's "name" property (stringified).
    pub fn name(self) -> Result<&'a str, AdtError> {
        self.prop("name")?.str()
    }

    /// Find a direct child by its "name" property.
    /// Accepts both exact match and `"<name>@<unit>"` style addressed names
    /// (common in the ADT; e.g. look up `uart0` even if the actual name
    /// on disk is `uart0@235200000`).
    pub fn subnode(self, wanted: &str) -> Result<Node<'a>, AdtError> {
        for child in self.children() {
            if let Ok(got) = child.name() {
                if got == wanted {
                    return Ok(child);
                }
                // Match `"name@unit"` when caller asked for `"name"`.
                if let Some((head, _)) = got.split_once('@') {
                    if head == wanted {
                        return Ok(child);
                    }
                }
            }
        }
        Err(AdtError::NotFound)
    }

    /// True if this node's `compatible` property contains `compat`
    /// (the property is a sequence of nul-terminated strings).
    pub fn is_compatible(self, compat: &str) -> bool {
        let Ok(p) = self.prop("compatible") else { return false; };
        for s in p.strings() {
            if s == compat {
                return true;
            }
        }
        false
    }

    /// Resolve the `idx`-th (address, size) pair from this node's `reg`
    /// property, translating through every ancestor's `ranges` property
    /// to produce an absolute physical address.
    ///
    /// Caller provides `path` — the sequence of ancestors from root to
    /// this node — because the translation needs to look up each
    /// ancestor's `#address-cells` / `#size-cells` / `ranges`.
    pub fn reg_absolute(
        self,
        root_to_here: &[Node<'a>],
        idx: usize,
    ) -> Result<(u64, u64), AdtError> {
        if root_to_here.is_empty() {
            return Err(AdtError::BadPath);
        }

        // Parent of this node is second-to-last in the path; if we're
        // looking at the root, there is no parent and reg is meaningless.
        if root_to_here.len() < 2 {
            return Err(AdtError::BadPath);
        }
        let mut cursor = root_to_here.len() - 1;
        let mut node = root_to_here[cursor];
        let mut parent = root_to_here[cursor - 1];

        let mut addr_cells = parent.prop("#address-cells")?.u32()? as usize;
        let mut size_cells = parent.prop("#size-cells")?.u32()? as usize;
        if !(1..=2).contains(&addr_cells) || size_cells > 2 {
            return Err(AdtError::BadCells);
        }

        let reg = node.prop("reg")?;
        let one = 4 * (addr_cells + size_cells);
        let want = idx.checked_add(1).and_then(|n| n.checked_mul(one))
            .ok_or(AdtError::BadLength)?;
        if reg.size() < want {
            return Err(AdtError::BadLength);
        }
        let base = idx * one;
        let val = reg.value()?;
        let mut addr = read_cells(&val[base..base + 4 * addr_cells])?;
        let size = read_cells(&val[base + 4 * addr_cells..base + one])?;

        // Walk up, translating through `ranges` at each step.
        while cursor > 0 {
            cursor -= 1;
            node = parent;
            if cursor == 0 {
                break;
            }
            parent = root_to_here[cursor - 1];

            // `ranges` is optional — if absent, the child's addresses
            // ARE the parent's addresses (1:1 mapping). Skip this level.
            let Ok(ranges_prop) = node.prop("ranges") else { continue; };
            let ranges = ranges_prop.value()?;
            // Empty `ranges` also means identity mapping.
            if ranges.is_empty() { continue; }

            let paddr_cells = parent.prop("#address-cells")?.u32()? as usize;
            if !(1..=2).contains(&paddr_cells) {
                return Err(AdtError::BadCells);
            }
            let entry = 4 * (paddr_cells + addr_cells + size_cells);
            if entry == 0 || ranges.len() % entry != 0 {
                return Err(AdtError::BadLength);
            }
            let mut found = false;
            for chunk in ranges.chunks_exact(entry) {
                let child_addr = read_cells(&chunk[..4 * addr_cells])?;
                let parent_addr = read_cells(&chunk[4 * addr_cells..4 * (addr_cells + paddr_cells)])?;
                let child_size = read_cells(&chunk[4 * (addr_cells + paddr_cells)..])?;
                if addr >= child_addr && addr.saturating_add(size) <= child_addr.saturating_add(child_size) {
                    addr = addr.wrapping_sub(child_addr).wrapping_add(parent_addr);
                    found = true;
                    break;
                }
            }
            if !found {
                // No range covered us — stop walking and return what we
                // have so far (might still be usable in identity regions).
                break;
            }
            addr_cells = paddr_cells;
            size_cells = parent.prop("#size-cells")?.u32()? as usize;
        }

        Ok((addr, size))
    }
}

/// Decode a little-endian u32 / u64 from an `addr-cells`-sized slice
/// (either 4 bytes = u32, or 8 bytes = u64).
fn read_cells(src: &[u8]) -> Result<u64, AdtError> {
    match src.len() {
        0 => Ok(0),
        4 => {
            let b: [u8; 4] = src.try_into().map_err(|_| AdtError::BadLength)?;
            Ok(u32::from_le_bytes(b) as u64)
        }
        8 => {
            // ADT stores 64-bit values as two little-endian u32s,
            // low word first.
            let lo: [u8; 4] = src[..4].try_into().map_err(|_| AdtError::BadLength)?;
            let hi: [u8; 4] = src[4..].try_into().map_err(|_| AdtError::BadLength)?;
            Ok(u32::from_le_bytes(lo) as u64 | ((u32::from_le_bytes(hi) as u64) << 32))
        }
        _ => Err(AdtError::BadLength),
    }
}

// ─── Property API ───────────────────────────────────────────────────

#[derive(Copy, Clone)]
pub struct Property<'a> {
    adt: Adt<'a>,
    /// Byte offset of the property HEADER (start of the 32-byte name).
    offset: usize,
    /// Cached size (the 30-bit real size, with flag bits stripped).
    size: usize,
    /// Byte offset of the value bytes.
    value_off: usize,
}

impl<'a> Property<'a> {
    /// Property name (nul-terminated field decoded as UTF-8; non-UTF-8
    /// returns empty string).
    pub fn name(self) -> &'a str {
        let end = core::cmp::min(self.offset + 32, self.adt.blob.len());
        let name_bytes = &self.adt.blob[self.offset..end];
        let nul = name_bytes.iter().position(|&b| b == 0).unwrap_or(name_bytes.len());
        core::str::from_utf8(&name_bytes[..nul]).unwrap_or("")
    }

    /// Raw byte slice of the property value.
    pub fn value(self) -> Result<&'a [u8], AdtError> {
        self.adt.prop_value(self.value_off, self.size)
    }

    /// Declared size in bytes.
    pub fn size(self) -> usize {
        self.size
    }

    /// As a single u32 (fails if size != 4).
    pub fn u32(self) -> Result<u32, AdtError> {
        if self.size != 4 {
            return Err(AdtError::BadLength);
        }
        let v = self.value()?;
        let b: [u8; 4] = v.try_into().map_err(|_| AdtError::BadLength)?;
        Ok(u32::from_le_bytes(b))
    }

    /// As a single u64 (fails if size != 8).
    pub fn u64(self) -> Result<u64, AdtError> {
        if self.size != 8 {
            return Err(AdtError::BadLength);
        }
        let v = self.value()?;
        let b: [u8; 8] = v.try_into().map_err(|_| AdtError::BadLength)?;
        Ok(u64::from_le_bytes(b))
    }

    /// As a UTF-8 string (takes bytes up to first nul). Fails if the
    /// prefix isn't valid UTF-8.
    pub fn str(self) -> Result<&'a str, AdtError> {
        let v = self.value()?;
        let nul = v.iter().position(|&b| b == 0).unwrap_or(v.len());
        core::str::from_utf8(&v[..nul]).map_err(|_| AdtError::NotString)
    }

    /// Iterate a property that's a concatenation of nul-terminated
    /// strings (the `compatible` property shape).
    pub fn strings(self) -> StringIter<'a> {
        StringIter {
            remaining: self.value().unwrap_or(&[]),
        }
    }
}

// ─── Iterators ──────────────────────────────────────────────────────

pub struct PropIter<'a> {
    adt: Adt<'a>,
    offset: usize,
    remaining: u32,
}
impl<'a> Iterator for PropIter<'a> {
    type Item = Property<'a>;
    fn next(&mut self) -> Option<Property<'a>> {
        if self.remaining == 0 { return None; }
        self.remaining -= 1;
        let (ph, value_off, next) = self.adt.read_prop_hdr(self.offset).ok()?;
        let p = Property {
            adt: self.adt,
            offset: self.offset,
            size: ph.size,
            value_off,
        };
        self.offset = next;
        Some(p)
    }
}

pub struct ChildIter<'a> {
    adt: Adt<'a>,
    offset: usize,
    remaining: u32,
}
impl<'a> Iterator for ChildIter<'a> {
    type Item = Node<'a>;
    fn next(&mut self) -> Option<Node<'a>> {
        if self.remaining == 0 { return None; }
        self.remaining -= 1;
        let node = Node { adt: self.adt, offset: self.offset };
        let sz = node.total_size().ok()?;
        self.offset = self.offset.checked_add(sz)?;
        Some(node)
    }
}

pub struct StringIter<'a> {
    remaining: &'a [u8],
}
impl<'a> Iterator for StringIter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        if self.remaining.is_empty() { return None; }
        let nul = self.remaining.iter().position(|&b| b == 0).unwrap_or(self.remaining.len());
        let s = core::str::from_utf8(&self.remaining[..nul]).ok()?;
        self.remaining = if nul < self.remaining.len() {
            &self.remaining[nul + 1..]
        } else {
            &[]
        };
        if s.is_empty() && self.remaining.is_empty() { None } else { Some(s) }
    }
}

// ─── Tests ──────────────────────────────────────────────────────────
//
// Building a tiny ADT blob by hand and parsing it. All tests use
// `no_std`-compatible constructs (no `Vec`) so they compile as part
// of the kernel crate.

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal ADT blob into `buf` starting at `off`. Returns
    /// the new offset. The layout produced here is the simplest
    /// possible: one root with a "name" property = "device-tree" and a
    /// single child node called "test-node" which has a "reg" property.
    fn build_min(buf: &mut [u8]) -> usize {
        // Root header: 1 property ("name"), 1 child.
        buf[0..4].copy_from_slice(&1u32.to_le_bytes());
        buf[4..8].copy_from_slice(&1u32.to_le_bytes());
        let mut off = 8;

        // Property "name" = "device-tree\0"
        let pname = b"name";
        buf[off..off + pname.len()].copy_from_slice(pname);
        for b in &mut buf[off + pname.len()..off + 32] { *b = 0; }
        off += 32;
        let val = b"device-tree\0";
        buf[off..off + 4].copy_from_slice(&(val.len() as u32).to_le_bytes());
        off += 4;
        buf[off..off + val.len()].copy_from_slice(val);
        off += ((val.len() + 3) / 4) * 4;

        // Child node header: 1 prop, 0 children.
        buf[off..off + 4].copy_from_slice(&1u32.to_le_bytes());
        buf[off + 4..off + 8].copy_from_slice(&0u32.to_le_bytes());
        off += 8;
        // Child's "name" property.
        let pname = b"name";
        buf[off..off + pname.len()].copy_from_slice(pname);
        for b in &mut buf[off + pname.len()..off + 32] { *b = 0; }
        off += 32;
        let val = b"test-node\0";
        buf[off..off + 4].copy_from_slice(&(val.len() as u32).to_le_bytes());
        off += 4;
        buf[off..off + val.len()].copy_from_slice(val);
        off += ((val.len() + 3) / 4) * 4;

        off
    }

    #[test]
    fn parse_minimal_tree() {
        let mut buf = [0u8; 256];
        let used = build_min(&mut buf);
        let adt = Adt::new(&buf[..used]).expect("adt parse");
        let root = adt.root().expect("root");
        assert_eq!(root.name().unwrap(), "device-tree");
        let child = root.subnode("test-node").expect("child");
        assert_eq!(child.name().unwrap(), "test-node");
    }

    #[test]
    fn reject_oob() {
        let buf = [0u8; 4]; // too short for even a node header
        assert!(Adt::new(&buf).is_err());
    }

    #[test]
    fn reject_malformed_prop_count() {
        let mut buf = [0u8; 32];
        // property_count = 99999 (over the 4096 cap) → BadOffset
        buf[0..4].copy_from_slice(&99999u32.to_le_bytes());
        buf[4..8].copy_from_slice(&0u32.to_le_bytes());
        assert!(Adt::new(&buf).is_err());
    }
}
