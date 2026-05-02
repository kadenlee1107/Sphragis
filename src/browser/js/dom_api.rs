#![allow(dead_code)]
// Bat_OS — JavaScript DOM API Integration
// Bridges JavaScript to the DOM tree, providing document/Element/Window objects.
// All no_std compatible with fixed-size arrays.

use super::value::{JsValue, ObjId, StringId};
use super::vm::{Vm, JsError};
use crate::browser::dom::{Document, NodeType, MAX_TEXT, MAX_NAME};

/// Global pointer to the active DOM document (set before script execution).
static mut DOM_DOC_PTR: *mut Document = core::ptr::null_mut();
/// Flag indicating DOM was mutated by JS and needs re-layout/repaint.
static mut DOM_DIRTY: bool = false;

/// Set the active DOM document pointer. Called from browser.rs before running scripts.
pub fn set_document(doc: &mut Document) {
    unsafe { DOM_DOC_PTR = doc as *mut Document; }
}

/// V8-ROOT-2: clear the cached DOM document pointer on cave switch. If we
/// don't, DOM_DOC_PTR still references cave A's Document (now invalid
/// kernel memory after destroy) when cave B's JS tries to access the DOM
/// — use-after-free into the old cave's heap.
pub fn reset_for_cave_switch() {
    unsafe {
        DOM_DOC_PTR = core::ptr::null_mut();
        DOM_DIRTY = false;
    }
}

/// Check and clear the dirty flag. Called after script execution.
pub fn take_dirty() -> bool {
    unsafe {
        let was = DOM_DIRTY;
        DOM_DIRTY = false;
        was
    }
}

fn mark_dirty() {
    unsafe { DOM_DIRTY = true; }
}

/// Get a reference to the live DOM document, if set.
fn get_doc() -> Option<&'static mut Document> {
    unsafe {
        if DOM_DOC_PTR.is_null() { None } else { Some(&mut *DOM_DOC_PTR) }
    }
}

/// Maximum DOM-to-JS bindings.
const MAX_DOM_BINDINGS: usize = 256;
/// Maximum event listeners.
const MAX_LISTENERS: usize = 128;
/// Maximum timers (setTimeout/setInterval).
const MAX_TIMERS: usize = 64;

/// Maps a DOM node index to a JS wrapper object.
#[derive(Clone, Copy)]
pub struct DomBinding {
    pub dom_node_idx: u32,  // index into DOM node array
    pub js_obj: ObjId,       // JS wrapper object
    pub active: bool,
}

/// An event listener registration.
#[derive(Clone, Copy)]
pub struct EventListener {
    pub target_dom_idx: u32,
    pub event_type: StringId,
    pub callback_obj: ObjId,
    pub active: bool,
}

/// A timer (setTimeout/setInterval).
#[derive(Clone, Copy)]
pub struct Timer {
    pub callback_obj: ObjId,
    pub delay_ms: u32,
    pub interval: bool,     // true for setInterval
    pub fire_at: u64,       // tick count when to fire
    pub active: bool,
    pub id: u32,
}

/// DOM integration state.
pub struct DomApi {
    pub bindings: [DomBinding; MAX_DOM_BINDINGS],
    pub binding_count: usize,
    pub listeners: [EventListener; MAX_LISTENERS],
    pub listener_count: usize,
    pub timers: [Timer; MAX_TIMERS],
    pub timer_count: usize,
    pub next_timer_id: u32,
    pub dom_dirty: bool,
}

impl DomApi {
    pub const fn new() -> Self {
        DomApi {
            bindings: [DomBinding { dom_node_idx: 0, js_obj: ObjId::NULL, active: false }; MAX_DOM_BINDINGS],
            binding_count: 0,
            listeners: [EventListener { target_dom_idx: 0, event_type: StringId::EMPTY, callback_obj: ObjId::NULL, active: false }; MAX_LISTENERS],
            listener_count: 0,
            timers: [Timer { callback_obj: ObjId::NULL, delay_ms: 0, interval: false, fire_at: 0, active: false, id: 0 }; MAX_TIMERS],
            timer_count: 0,
            next_timer_id: 1,
            dom_dirty: false,
        }
    }

    /// Get or create a JS wrapper for a DOM node index.
    /// The wrapper gets native DOM methods bound to it (appendChild, setAttribute, etc.)
    pub fn get_or_create_wrapper(&mut self, dom_idx: u32, heap: &mut super::object::JsHeap) -> ObjId {
        // Check if we already have a binding
        for i in 0..self.binding_count {
            if self.bindings[i].active && self.bindings[i].dom_node_idx == dom_idx {
                return self.bindings[i].js_obj;
            }
        }
        // Create new binding
        if self.binding_count >= MAX_DOM_BINDINGS {
            return ObjId::NULL;
        }
        let obj = heap.alloc_object();
        self.bindings[self.binding_count] = DomBinding {
            dom_node_idx: dom_idx,
            js_obj: obj,
            active: true,
        };
        self.binding_count += 1;
        obj
    }

    /// Reset the DOM binding state (for new navigations).
    pub fn reset(&mut self) {
        self.binding_count = 0;
        self.listener_count = 0;
        self.dom_dirty = false;
    }

    /// Look up DOM node index for a JS wrapper object.
    pub fn get_dom_idx(&self, obj: ObjId) -> Option<u32> {
        for i in 0..self.binding_count {
            if self.bindings[i].active && self.bindings[i].js_obj.0 == obj.0 {
                return Some(self.bindings[i].dom_node_idx);
            }
        }
        None
    }

    /// Add an event listener.
    pub fn add_event_listener(&mut self, dom_idx: u32, event_type: StringId, callback: ObjId) {
        if self.listener_count >= MAX_LISTENERS { return; }
        self.listeners[self.listener_count] = EventListener {
            target_dom_idx: dom_idx,
            event_type,
            callback_obj: callback,
            active: true,
        };
        self.listener_count += 1;
    }

    /// Remove an event listener.
    pub fn remove_event_listener(&mut self, dom_idx: u32, event_type: StringId, callback: ObjId) {
        for i in 0..self.listener_count {
            if self.listeners[i].active
                && self.listeners[i].target_dom_idx == dom_idx
                && self.listeners[i].event_type.0 == event_type.0
                && self.listeners[i].callback_obj.0 == callback.0
            {
                self.listeners[i].active = false;
                return;
            }
        }
    }

    /// Create a timer. Returns timer ID.
    pub fn create_timer(&mut self, callback: ObjId, delay_ms: u32, interval: bool, current_tick: u64) -> u32 {
        if self.timer_count >= MAX_TIMERS { return 0; }
        let id = self.next_timer_id;
        self.next_timer_id += 1;
        self.timers[self.timer_count] = Timer {
            callback_obj: callback,
            delay_ms,
            interval,
            fire_at: current_tick + delay_ms as u64,
            active: true,
            id,
        };
        self.timer_count += 1;
        id
    }

    /// Clear a timer by ID.
    pub fn clear_timer(&mut self, id: u32) {
        for i in 0..self.timer_count {
            if self.timers[i].active && self.timers[i].id == id {
                self.timers[i].active = false;
                return;
            }
        }
    }

    /// Mark DOM as dirty (needs re-layout/repaint).
    pub fn set_dirty(&mut self) {
        self.dom_dirty = true;
    }

    /// Check and clear dirty flag.
    pub fn take_dirty(&mut self) -> bool {
        let was = self.dom_dirty;
        self.dom_dirty = false;
        was
    }
}

// ─── Native DOM functions ───

pub fn dom_get_element_by_id(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::NULL); }
    let id_val = vm.stack[args_start];
    if !id_val.is_string() { return Ok(JsValue::NULL); }
    let id_bytes = vm.strings.get(id_val.as_str_id());

    // Copy id_bytes to a local buffer since we'll reborrow vm
    let mut id_buf = [0u8; 128];
    let id_len = id_bytes.len().min(128);
    id_buf[..id_len].copy_from_slice(&id_bytes[..id_len]);

    if let Some(doc) = get_doc() {
        for i in 0..doc.node_count {
            if doc.nodes[i].node_type == NodeType::Element {
                if let Some(attr_val) = doc.nodes[i].get_attr("id") {
                    if attr_val.as_bytes() == &id_buf[..id_len] {
                        // Found — get tag name before borrowing vm.dom
                        let mut tag_buf = [0u8; MAX_NAME];
                        let tag_len = doc.nodes[i].tag_len;
                        tag_buf[..tag_len].copy_from_slice(&doc.nodes[i].tag[..tag_len]);

                        let obj = vm.dom.get_or_create_wrapper(i as u32, &mut vm.heap);
                        let tag_name_sid = vm.strings.intern(b"tagName");
                        let tag_val = vm.strings.intern(&tag_buf[..tag_len]);
                        vm.heap.set_prop(obj, tag_name_sid, JsValue::from_str(tag_val));
                        install_element_methods(vm, obj);
                        return Ok(JsValue::from_obj(obj));
                    }
                }
            }
        }
    }
    Ok(JsValue::NULL)
}

pub fn dom_query_selector(_vm: &mut Vm, _args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::NULL); }
    Ok(JsValue::NULL)
}

pub fn dom_query_selector_all(vm: &mut Vm, _args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::NULL); }
    let arr = vm.heap.alloc_array(0);
    Ok(JsValue::from_obj(arr))
}

pub fn dom_create_element(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::NULL); }
    let tag_val = vm.stack[args_start];

    // Copy tag bytes to local buffer to avoid borrow conflict
    let mut tag_buf = [0u8; MAX_NAME];
    let tag_len;
    if tag_val.is_string() {
        let tb = vm.strings.get(tag_val.as_str_id());
        // STUMP #111 (audit M-tag-validation): reject tag names with
        // control bytes, NUL, or whitespace. createElement("div\x00script")
        // could store a node whose `tag_str()` is "div\x00script" — most
        // downstream code uses byte-equality so this doesn't immediately
        // exploit, but ANY future code that does any kind of split-on-NUL
        // or printable-prefix match would create a defense bypass. Reject
        // at the source. Empty tag also rejected.
        if tb.is_empty() {
            return Ok(JsValue::NULL);
        }
        for &b in tb {
            if b < 0x20 || b == 0x7F || b == b' ' || b == b'<' || b == b'>' || b == b'/' {
                return Ok(JsValue::NULL);
            }
        }
        tag_len = tb.len().min(MAX_NAME);
        tag_buf[..tag_len].copy_from_slice(&tb[..tag_len]);
    } else {
        tag_buf[..3].copy_from_slice(b"div");
        tag_len = 3;
    }

    if let Some(doc) = get_doc() {
        if let Some(idx) = doc.create_element(&tag_buf[..tag_len]) {
            let mut real_tag = [0u8; MAX_NAME];
            let rtl = doc.nodes[idx].tag_len;
            real_tag[..rtl].copy_from_slice(&doc.nodes[idx].tag[..rtl]);

            let obj = vm.dom.get_or_create_wrapper(idx as u32, &mut vm.heap);
            let tag_name_sid = vm.strings.intern(b"tagName");
            let tag_str_sid = vm.strings.intern(&real_tag[..rtl]);
            vm.heap.set_prop(obj, tag_name_sid, JsValue::from_str(tag_str_sid));
            let child_name = vm.strings.intern(b"children");
            let children = vm.heap.alloc_array(0);
            vm.heap.set_prop(obj, child_name, JsValue::from_obj(children));
            let text_name = vm.strings.intern(b"textContent");
            vm.heap.set_prop(obj, text_name, JsValue::from_str(StringId::EMPTY));
            install_element_methods(vm, obj);
            return Ok(JsValue::from_obj(obj));
        }
    }
    // Fallback: create a JS-only wrapper if no DOM available
    let elem = vm.heap.alloc_object();
    let tag_name = vm.strings.intern(b"tagName");
    if tag_val.is_string() {
        vm.heap.set_prop(elem, tag_name, tag_val);
    }
    let child_name = vm.strings.intern(b"children");
    let children = vm.heap.alloc_array(0);
    vm.heap.set_prop(elem, child_name, JsValue::from_obj(children));
    let text_name = vm.strings.intern(b"textContent");
    vm.heap.set_prop(elem, text_name, JsValue::from_str(StringId::EMPTY));
    install_element_methods(vm, elem);
    Ok(JsValue::from_obj(elem))
}

pub fn dom_create_text_node(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    // Copy text to a local buffer
    let mut text_buf = [0u8; MAX_TEXT];
    let text_len;
    if argc > 0 && vm.stack[args_start].is_string() {
        let tb = vm.strings.get(vm.stack[args_start].as_str_id());
        text_len = tb.len().min(MAX_TEXT);
        text_buf[..text_len].copy_from_slice(&tb[..text_len]);
    } else {
        text_len = 0;
    }

    if let Some(doc) = get_doc() {
        if let Some(idx) = doc.create_text(&text_buf[..text_len]) {
            let obj = vm.dom.get_or_create_wrapper(idx as u32, &mut vm.heap);
            let text_name = vm.strings.intern(b"textContent");
            let text_sid = vm.strings.intern(&text_buf[..text_len]);
            vm.heap.set_prop(obj, text_name, JsValue::from_str(text_sid));
            let node_type_name = vm.strings.intern(b"nodeType");
            vm.heap.set_prop(obj, node_type_name, JsValue::from_i32(3));
            return Ok(JsValue::from_obj(obj));
        }
    }
    // Fallback
    let text_node = vm.heap.alloc_object();
    let text_name = vm.strings.intern(b"textContent");
    let text = if argc > 0 && vm.stack[args_start].is_string() {
        vm.stack[args_start]
    } else {
        JsValue::from_str(StringId::EMPTY)
    };
    vm.heap.set_prop(text_node, text_name, text);
    let node_type_name = vm.strings.intern(b"nodeType");
    vm.heap.set_prop(text_node, node_type_name, JsValue::from_i32(3));
    Ok(JsValue::from_obj(text_node))
}

pub fn dom_append_child(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 || argc == 0 { return Ok(JsValue::UNDEFINED); }
    let this_val = vm.stack[args_start - 1];
    let child = vm.stack[args_start];
    if this_val.is_object() && child.is_object() {
        // Try to modify real DOM tree
        let parent_dom = vm.dom.get_dom_idx(this_val.as_obj());
        let child_dom = vm.dom.get_dom_idx(child.as_obj());
        if let (Some(pidx), Some(cidx)) = (parent_dom, child_dom) {
            if let Some(doc) = get_doc() {
                doc.append_child(pidx as usize, cidx as usize);
                mark_dirty();
            }
        }
        // Also maintain JS-side children array
        let child_name = vm.strings.intern(b"children");
        let children = vm.heap.get_prop(this_val.as_obj(), child_name);
        if children.is_object() {
            vm.heap.array_push(children.as_obj(), child);
        }
        let parent_name = vm.strings.intern(b"parentElement");
        vm.heap.set_prop(child.as_obj(), parent_name, this_val);
    }
    Ok(child)
}

pub fn dom_remove_child(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::UNDEFINED); }
    let child = vm.stack[args_start];
    // Remove parent reference
    if child.is_object() {
        let parent_name = vm.strings.intern(b"parentElement");
        vm.heap.set_prop(child.as_obj(), parent_name, JsValue::NULL);
    }
    Ok(child)
}

pub fn dom_set_attribute(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 || argc < 2 { return Ok(JsValue::UNDEFINED); }
    let this_val = vm.stack[args_start - 1];
    if this_val.is_object() {
        let name = vm.stack[args_start];
        let value = vm.stack[args_start + 1];
        if name.is_string() {
            // Set on real DOM node if bound
            if let Some(dom_idx) = vm.dom.get_dom_idx(this_val.as_obj()) {
                let attr_name = vm.strings.get(name.as_str_id());
                // STUMP #111 (audit H022): reject attribute names
                // containing NUL or any control character. Pre-fix
                // setAttribute("on\x00load", "alert(1)") stored the
                // raw bytes; depending on downstream consumers the
                // NUL could be stripped, leaving "onload" — defense
                // bypass. Empty name also rejected.
                if attr_name.is_empty() {
                    return Ok(JsValue::UNDEFINED);
                }
                for &b in attr_name {
                    if b < 0x20 || b == 0x7F {
                        return Ok(JsValue::UNDEFINED);
                    }
                }
                let attr_val = if value.is_string() {
                    vm.strings.get(value.as_str_id())
                } else {
                    b"" as &[u8]
                };
                if let Some(doc) = get_doc() {
                    let node = &mut doc.nodes[dom_idx as usize];
                    if node.attr_count < crate::browser::dom::MAX_ATTRS {
                        // Check if attribute already exists, update it
                        let mut found = false;
                        for ai in 0..node.attr_count {
                            if &node.attrs[ai].name[..node.attrs[ai].name_len] == attr_name {
                                node.attrs[ai].set_value(attr_val);
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            let ai = node.attr_count;
                            node.attrs[ai].set_name(attr_name);
                            node.attrs[ai].set_value(attr_val);
                            node.attr_count += 1;
                        }
                        mark_dirty();
                    }
                }
            }
            // Also set on JS wrapper
            vm.heap.set_prop(this_val.as_obj(), name.as_str_id(), value);
        }
    }
    Ok(JsValue::UNDEFINED)
}

pub fn dom_get_attribute(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 || argc == 0 { return Ok(JsValue::NULL); }
    let this_val = vm.stack[args_start - 1];
    if this_val.is_object() {
        let name = vm.stack[args_start];
        if name.is_string() {
            // Try to read from real DOM node
            if let Some(dom_idx) = vm.dom.get_dom_idx(this_val.as_obj()) {
                let attr_name_bytes = vm.strings.get(name.as_str_id());
                if let Some(doc) = get_doc() {
                    let node = &doc.nodes[dom_idx as usize];
                    let attr_name_str = unsafe { core::str::from_utf8_unchecked(attr_name_bytes) };
                    if let Some(val_str) = node.get_attr(attr_name_str) {
                        let sid = vm.strings.intern(val_str.as_bytes());
                        return Ok(JsValue::from_str(sid));
                    }
                }
            }
            // Fallback to JS property
            let val = vm.heap.get_prop(this_val.as_obj(), name.as_str_id());
            return Ok(val);
        }
    }
    Ok(JsValue::NULL)
}

pub fn dom_add_event_listener(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 || argc < 2 { return Ok(JsValue::UNDEFINED); }
    // Store event type and callback on the object
    let this_val = vm.stack[args_start - 1];
    if this_val.is_object() {
        let event_type = vm.stack[args_start];
        let callback = vm.stack[args_start + 1];
        if event_type.is_string() && callback.is_object() {
            // Store as __evt_<type> property
            let mut key = [0u8; 64];
            key[0] = b'_'; key[1] = b'_'; key[2] = b'e'; key[3] = b'v'; key[4] = b't'; key[5] = b'_';
            let evtname = vm.strings.get(event_type.as_str_id());
            let elen = evtname.len().min(58);
            key[6..6 + elen].copy_from_slice(&evtname[..elen]);
            let key_sid = vm.strings.intern(&key[..6 + elen]);
            vm.heap.set_prop(this_val.as_obj(), key_sid, callback);
        }
    }
    Ok(JsValue::UNDEFINED)
}

pub fn dom_remove_event_listener(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 || argc < 2 { return Ok(JsValue::UNDEFINED); }
    let this_val = vm.stack[args_start - 1];
    if this_val.is_object() {
        let event_type = vm.stack[args_start];
        if event_type.is_string() {
            let mut key = [0u8; 64];
            key[0] = b'_'; key[1] = b'_'; key[2] = b'e'; key[3] = b'v'; key[4] = b't'; key[5] = b'_';
            let evtname = vm.strings.get(event_type.as_str_id());
            let elen = evtname.len().min(58);
            key[6..6 + elen].copy_from_slice(&evtname[..elen]);
            let key_sid = vm.strings.intern(&key[..6 + elen]);
            vm.heap.set_prop(this_val.as_obj(), key_sid, JsValue::UNDEFINED);
        }
    }
    Ok(JsValue::UNDEFINED)
}

pub fn dom_set_timeout(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if argc == 0 { return Ok(JsValue::from_i32(0)); }
    // Store the callback reference; actual timer scheduling is external
    let _callback = vm.stack[args_start];
    let _delay = if argc > 1 { vm.stack[args_start + 1].to_i32() } else { 0 };
    // Return a dummy timer ID
    Ok(JsValue::from_i32(1))
}

pub fn dom_set_interval(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    dom_set_timeout(vm, args_start, argc)
}

pub fn dom_clear_timeout(_vm: &mut Vm, _args_start: usize, _argc: usize) -> Result<JsValue, JsError> {
    Ok(JsValue::UNDEFINED)
}

pub fn dom_clear_interval(_vm: &mut Vm, _args_start: usize, _argc: usize) -> Result<JsValue, JsError> {
    Ok(JsValue::UNDEFINED)
}

pub fn dom_get_bounding_rect(vm: &mut Vm, _args_start: usize, _argc: usize) -> Result<JsValue, JsError> {
    let rect = vm.heap.alloc_object();
    let x_name = vm.strings.intern(b"x");
    let y_name = vm.strings.intern(b"y");
    let width_name = vm.strings.intern(b"width");
    let height_name = vm.strings.intern(b"height");
    vm.heap.set_prop(rect, x_name, JsValue::from_i32(0));
    vm.heap.set_prop(rect, y_name, JsValue::from_i32(0));
    vm.heap.set_prop(rect, width_name, JsValue::from_i32(0));
    vm.heap.set_prop(rect, height_name, JsValue::from_i32(0));
    Ok(JsValue::from_obj(rect))
}

/// Install common element methods (appendChild, setAttribute, etc.) on a JS wrapper object.
pub fn install_element_methods(vm: &mut Vm, obj: ObjId) {
    let ac_fn = vm.make_native_function(dom_append_child);
    let ac_name = vm.strings.intern(b"appendChild");
    vm.heap.set_prop(obj, ac_name, JsValue::from_obj(ac_fn));

    let rc_fn = vm.make_native_function(dom_remove_child);
    let rc_name = vm.strings.intern(b"removeChild");
    vm.heap.set_prop(obj, rc_name, JsValue::from_obj(rc_fn));

    let sa_fn = vm.make_native_function(dom_set_attribute);
    let sa_name = vm.strings.intern(b"setAttribute");
    vm.heap.set_prop(obj, sa_name, JsValue::from_obj(sa_fn));

    let ga_fn = vm.make_native_function(dom_get_attribute);
    let ga_name = vm.strings.intern(b"getAttribute");
    vm.heap.set_prop(obj, ga_name, JsValue::from_obj(ga_fn));

    let ael_fn = vm.make_native_function(dom_add_event_listener);
    let ael_name = vm.strings.intern(b"addEventListener");
    vm.heap.set_prop(obj, ael_name, JsValue::from_obj(ael_fn));

    let rel_fn = vm.make_native_function(dom_remove_event_listener);
    let rel_name = vm.strings.intern(b"removeEventListener");
    vm.heap.set_prop(obj, rel_name, JsValue::from_obj(rel_fn));

    let gbr_fn = vm.make_native_function(dom_get_bounding_rect);
    let gbr_name = vm.strings.intern(b"getBoundingClientRect");
    vm.heap.set_prop(obj, gbr_name, JsValue::from_obj(gbr_fn));
}

/// Register DOM globals on the VM.
pub fn register_dom_globals(vm: &mut Vm) {
    // document object
    let doc = vm.heap.alloc_object();

    let gid_fn = vm.make_native_function(dom_get_element_by_id);
    let gid_name = vm.strings.intern(b"getElementById");
    vm.heap.set_prop(doc, gid_name, JsValue::from_obj(gid_fn));

    let qs_fn = vm.make_native_function(dom_query_selector);
    let qs_name = vm.strings.intern(b"querySelector");
    vm.heap.set_prop(doc, qs_name, JsValue::from_obj(qs_fn));

    let qsa_fn = vm.make_native_function(dom_query_selector_all);
    let qsa_name = vm.strings.intern(b"querySelectorAll");
    vm.heap.set_prop(doc, qsa_name, JsValue::from_obj(qsa_fn));

    let ce_fn = vm.make_native_function(dom_create_element);
    let ce_name = vm.strings.intern(b"createElement");
    vm.heap.set_prop(doc, ce_name, JsValue::from_obj(ce_fn));

    let ctn_fn = vm.make_native_function(dom_create_text_node);
    let ctn_name = vm.strings.intern(b"createTextNode");
    vm.heap.set_prop(doc, ctn_name, JsValue::from_obj(ctn_fn));

    // document.body — bind to real DOM body node if available
    let body = vm.heap.alloc_object();
    let children = vm.heap.alloc_array(0);
    let child_name = vm.strings.intern(b"children");
    vm.heap.set_prop(body, child_name, JsValue::from_obj(children));
    // Install element methods on body
    install_element_methods(vm, body);
    let body_name = vm.strings.intern(b"body");
    vm.heap.set_prop(doc, body_name, JsValue::from_obj(body));

    // Also install appendChild on document itself
    install_element_methods(vm, doc);

    let doc_name = vm.strings.intern(b"document");
    vm.set_global(doc_name, JsValue::from_obj(doc));

    // window object
    let win = vm.heap.alloc_object();

    let st_fn = vm.make_native_function(dom_set_timeout);
    let st_name = vm.strings.intern(b"setTimeout");
    vm.heap.set_prop(win, st_name, JsValue::from_obj(st_fn));

    let si_fn = vm.make_native_function(dom_set_interval);
    let si_name = vm.strings.intern(b"setInterval");
    vm.heap.set_prop(win, si_name, JsValue::from_obj(si_fn));

    let ct_fn = vm.make_native_function(dom_clear_timeout);
    let ct_name = vm.strings.intern(b"clearTimeout");
    vm.heap.set_prop(win, ct_name, JsValue::from_obj(ct_fn));

    let ci_fn = vm.make_native_function(dom_clear_interval);
    let ci_name = vm.strings.intern(b"clearInterval");
    vm.heap.set_prop(win, ci_name, JsValue::from_obj(ci_fn));

    let alert_fn = vm.make_native_function(super::vm::native_alert_for_window);
    let alert_name = vm.strings.intern(b"alert");
    vm.heap.set_prop(win, alert_name, JsValue::from_obj(alert_fn));

    // window.location
    let location = vm.heap.alloc_object();
    let href_name = vm.strings.intern(b"href");
    let href_val = vm.strings.intern(b"about:blank");
    vm.heap.set_prop(location, href_name, JsValue::from_str(href_val));
    let loc_name = vm.strings.intern(b"location");
    vm.heap.set_prop(win, loc_name, JsValue::from_obj(location));

    let win_name = vm.strings.intern(b"window");
    vm.set_global(win_name, JsValue::from_obj(win));

    // Also register setTimeout/setInterval as globals (accessible without window.)
    vm.set_global(st_name, JsValue::from_obj(st_fn));
    vm.set_global(si_name, JsValue::from_obj(si_fn));
    vm.set_global(ct_name, JsValue::from_obj(ct_fn));
    vm.set_global(ci_name, JsValue::from_obj(ci_fn));
}
