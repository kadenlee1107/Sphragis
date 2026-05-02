#![allow(dead_code)]
// Bat_OS — JavaScript Object Heap
// Arena-allocated objects with property storage.
// No hash maps — uses linear property arrays (fast for objects with < 32 props).

use super::value::{JsValue, ObjId, StringId};

/// Maximum objects in the heap.
const MAX_OBJECTS: usize = 4096;
/// Maximum total property slots across all objects.
const MAX_PROPS: usize = 32768;
/// Maximum properties per object.
const MAX_OBJ_PROPS: usize = 32;

/// Object flag bits.
pub struct ObjFlags;
impl ObjFlags {
    pub const NONE: u8     = 0;
    pub const ARRAY: u8    = 0x01;  // Object is an array
    pub const FUNCTION: u8 = 0x02;  // Object is a user-defined function
    pub const NATIVE: u8   = 0x04;  // Object is a native function
    pub const FROZEN: u8   = 0x08;  // Object is frozen (no mutations)
    pub const SEALED: u8   = 0x10;  // Object is sealed (no new props)
}

/// A property slot: name + value.
#[derive(Clone, Copy)]
struct PropSlot {
    name: StringId,
    value: JsValue,
}

/// Object header in the arena.
#[derive(Clone, Copy)]
struct JsObject {
    active: bool,
    flags: u8,
    prototype: ObjId,
    prop_start: u32,     // index into props array
    prop_count: u16,     // current number of properties
    prop_capacity: u16,  // allocated prop slots
    // For arrays: stored length (may differ from dense element count)
    array_len: u32,
    // For native functions: index into native function table
    native_idx: u16,
    // For user functions: index into proto table
    func_proto_idx: u16,
}

/// The object heap: arena of objects + property storage.
pub struct JsHeap {
    objects: [JsObject; MAX_OBJECTS],
    object_count: usize,
    props: [PropSlot; MAX_PROPS],
    prop_count: usize,
}

impl JsHeap {
    pub const fn new() -> Self {
        JsHeap {
            objects: [JsObject {
                active: false,
                flags: 0,
                prototype: ObjId::NULL,
                prop_start: 0,
                prop_count: 0,
                prop_capacity: 0,
                array_len: 0,
                native_idx: 0,
                func_proto_idx: 0,
            }; MAX_OBJECTS],
            object_count: 0,
            props: [PropSlot { name: StringId::EMPTY, value: JsValue::UNDEFINED }; MAX_PROPS],
            prop_count: 0,
        }
    }

    /// Allocate a plain object.
    pub fn alloc_object(&mut self) -> ObjId {
        self.alloc_with_flags(ObjFlags::NONE, 8)
    }

    /// Allocate an array with initial capacity.
    pub fn alloc_array(&mut self, len: u32) -> ObjId {
        let id = self.alloc_with_flags(ObjFlags::ARRAY, len.max(8) as u16);
        if !id.is_null() {
            self.objects[id.0 as usize].array_len = len;
        }
        id
    }

    /// Allocate a native function object.
    pub fn alloc_native_function(&mut self, native_idx: u16) -> ObjId {
        let id = self.alloc_with_flags(ObjFlags::NATIVE, 4);
        if !id.is_null() {
            self.objects[id.0 as usize].native_idx = native_idx;
        }
        id
    }

    fn alloc_with_flags(&mut self, flags: u8, capacity: u16) -> ObjId {
        // STUMP #111 (audit M-jsheap-saturation): one-shot audit when
        // either object table or prop arena saturates. Pre-fix scripts
        // that allocated >4096 objects (or >32768 props) just got
        // ObjId::NULL with no log entry — flooding behavior was
        // invisible. Two separate flags so we can distinguish which
        // table is exhausted.
        use core::sync::atomic::{AtomicBool, Ordering};
        if self.object_count >= MAX_OBJECTS {
            static FF_OBJ: AtomicBool = AtomicBool::new(false);
            if !FF_OBJ.swap(true, Ordering::AcqRel) {
                crate::security::audit::record(
                    crate::security::audit::Category::Script,
                    b"JS heap MAX_OBJECTS (4096) full - script may be flooding",
                );
                crate::drivers::uart::puts("[js] WARNING: object heap full - alloc returns null\n");
            }
            return ObjId::NULL;
        }
        let cap = (capacity as usize).min(MAX_OBJ_PROPS);
        if self.prop_count + cap > MAX_PROPS {
            static FF_PROP: AtomicBool = AtomicBool::new(false);
            if !FF_PROP.swap(true, Ordering::AcqRel) {
                crate::security::audit::record(
                    crate::security::audit::Category::Script,
                    b"JS heap MAX_PROPS (32768) full - script may be flooding",
                );
                crate::drivers::uart::puts("[js] WARNING: property arena full - alloc returns null\n");
            }
            return ObjId::NULL;
        }

        let idx = self.object_count;
        self.objects[idx] = JsObject {
            active: true,
            flags,
            prototype: ObjId::NULL,
            prop_start: self.prop_count as u32,
            prop_count: 0,
            prop_capacity: cap as u16,
            array_len: 0,
            native_idx: 0,
            func_proto_idx: 0,
        };
        self.prop_count += cap;
        self.object_count += 1;
        ObjId(idx as u32)
    }

    /// Get object flags.
    pub fn get_flags(&self, id: ObjId) -> u8 {
        let idx = id.0 as usize;
        if idx >= self.object_count { return 0; }
        self.objects[idx].flags
    }

    /// Get native function index.
    pub fn get_native_idx(&self, id: ObjId) -> u16 {
        let idx = id.0 as usize;
        if idx >= self.object_count { return 0; }
        self.objects[idx].native_idx
    }

    // ─── Property operations ───

    /// Get a named property from an object (own properties only).
    pub fn get_prop(&self, id: ObjId, name: StringId) -> JsValue {
        let idx = id.0 as usize;
        if idx >= self.object_count { return JsValue::UNDEFINED; }
        let obj = &self.objects[idx];
        let start = obj.prop_start as usize;
        for i in 0..obj.prop_count as usize {
            if self.props[start + i].name.0 == name.0 {
                return self.props[start + i].value;
            }
        }
        // Check prototype chain
        if !obj.prototype.is_null() {
            return self.get_prop(obj.prototype, name);
        }
        JsValue::UNDEFINED
    }

    /// Set a named property on an object.
    pub fn set_prop(&mut self, id: ObjId, name: StringId, value: JsValue) {
        let idx = id.0 as usize;
        if idx >= self.object_count { return; }

        let start = self.objects[idx].prop_start as usize;
        let count = self.objects[idx].prop_count as usize;
        let cap = self.objects[idx].prop_capacity as usize;

        // Check if property already exists
        for i in 0..count {
            if self.props[start + i].name.0 == name.0 {
                self.props[start + i].value = value;
                return;
            }
        }

        // Add new property if capacity allows
        if count < cap {
            self.props[start + count] = PropSlot { name, value };
            self.objects[idx].prop_count += 1;
        }
    }

    /// Check if object has own property.
    pub fn has_own_prop(&self, id: ObjId, name: StringId) -> bool {
        let idx = id.0 as usize;
        if idx >= self.object_count { return false; }
        let obj = &self.objects[idx];
        let start = obj.prop_start as usize;
        for i in 0..obj.prop_count as usize {
            if self.props[start + i].name.0 == name.0 {
                return true;
            }
        }
        false
    }

    /// Get all own property names.
    pub fn own_keys(&self, id: ObjId, out: &mut [StringId]) -> usize {
        let idx = id.0 as usize;
        if idx >= self.object_count { return 0; }
        let obj = &self.objects[idx];
        let start = obj.prop_start as usize;
        let count = obj.prop_count as usize;
        let n = count.min(out.len());
        for i in 0..n {
            out[i] = self.props[start + i].name;
        }
        n
    }

    // ─── Array operations ───

    /// Get array element by index (dense storage).
    pub fn array_get(&self, id: ObjId, index: u32) -> JsValue {
        let idx = id.0 as usize;
        if idx >= self.object_count { return JsValue::UNDEFINED; }
        let obj = &self.objects[idx];
        if index >= obj.prop_capacity as u32 { return JsValue::UNDEFINED; }
        let start = obj.prop_start as usize;
        self.props[start + index as usize].value
    }

    /// Set array element by index (dense storage).
    pub fn array_set(&mut self, id: ObjId, index: u32, value: JsValue) {
        let idx = id.0 as usize;
        if idx >= self.object_count { return; }
        let cap = self.objects[idx].prop_capacity as u32;
        if index >= cap { return; } // TODO: grow array

        let start = self.objects[idx].prop_start as usize;
        self.props[start + index as usize].value = value;

        // Update length if needed
        if index >= self.objects[idx].array_len {
            self.objects[idx].array_len = index + 1;
        }
    }

    /// Get array length.
    pub fn array_len(&self, id: ObjId) -> u32 {
        let idx = id.0 as usize;
        if idx >= self.object_count { return 0; }
        self.objects[idx].array_len
    }

    /// Push element to end of array.
    pub fn array_push(&mut self, id: ObjId, value: JsValue) -> u32 {
        let idx = id.0 as usize;
        if idx >= self.object_count { return 0; }
        let len = self.objects[idx].array_len;
        self.array_set(id, len, value);
        self.objects[idx].array_len
    }

    /// Set array length explicitly.
    pub fn set_array_len(&mut self, id: ObjId, len: u32) {
        let idx = id.0 as usize;
        if idx < self.object_count {
            self.objects[idx].array_len = len;
        }
    }

    /// Pop element from end of array.
    pub fn array_pop(&mut self, id: ObjId) -> JsValue {
        let idx = id.0 as usize;
        if idx >= self.object_count { return JsValue::UNDEFINED; }
        let len = self.objects[idx].array_len;
        if len == 0 { return JsValue::UNDEFINED; }
        let val = self.array_get(id, len - 1);
        self.objects[idx].array_len -= 1;
        val
    }

    /// Allocate a closure (user function) object.
    pub fn alloc_closure(&mut self, proto_idx: u16) -> ObjId {
        let id = self.alloc_with_flags(ObjFlags::FUNCTION, 16);
        if !id.is_null() {
            self.objects[id.0 as usize].func_proto_idx = proto_idx;
        }
        id
    }

    /// Get function prototype index.
    pub fn get_func_proto_idx(&self, id: ObjId) -> u16 {
        let idx = id.0 as usize;
        if idx >= self.object_count { return 0; }
        self.objects[idx].func_proto_idx
    }

    /// Set additional flags on an object.
    pub fn set_flags(&mut self, id: ObjId, flag: u8) {
        let idx = id.0 as usize;
        if idx < self.object_count {
            self.objects[idx].flags |= flag;
        }
    }

    /// Set object prototype.
    pub fn set_prototype(&mut self, id: ObjId, proto: ObjId) {
        let idx = id.0 as usize;
        if idx < self.object_count {
            self.objects[idx].prototype = proto;
        }
    }

    /// Get total allocated objects (for diagnostics).
    pub fn total_objects(&self) -> usize { self.object_count }

    /// Get total used property slots (for diagnostics).
    pub fn total_props(&self) -> usize { self.prop_count }
}
