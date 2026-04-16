// Bat_OS — JavaScript Built-in Methods
// Native implementations of Array, String, Object, Number, Date, Error
// prototype methods and static functions. All no_std compatible.

use super::value::{JsValue, ObjId, StringId};
use super::vm::{Vm, JsError};
use super::object::ObjFlags;

// ─── Array prototype methods ───

pub fn array_push(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    // 'this' is the last arg in method calls (pushed by SWAP after DUP)
    if args_start == 0 { return Ok(JsValue::UNDEFINED); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_object() { return Ok(JsValue::UNDEFINED); }
    let obj = this_val.as_obj();
    for i in 0..argc {
        vm.heap.array_push(obj, vm.stack[args_start + i]);
    }
    let len = vm.heap.array_len(obj);
    Ok(JsValue::from_i32(len as i32))
}

pub fn array_pop(vm: &mut Vm, args_start: usize, _argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::UNDEFINED); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_object() { return Ok(JsValue::UNDEFINED); }
    Ok(vm.heap.array_pop(this_val.as_obj()))
}

pub fn array_shift(vm: &mut Vm, args_start: usize, _argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::UNDEFINED); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_object() { return Ok(JsValue::UNDEFINED); }
    let obj = this_val.as_obj();
    let len = vm.heap.array_len(obj);
    if len == 0 { return Ok(JsValue::UNDEFINED); }
    let first = vm.heap.array_get(obj, 0);
    // Shift elements down
    for i in 1..len {
        let v = vm.heap.array_get(obj, i);
        vm.heap.array_set(obj, i - 1, v);
    }
    vm.heap.set_array_len(obj, len - 1);
    Ok(first)
}

pub fn array_unshift(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::UNDEFINED); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_object() { return Ok(JsValue::UNDEFINED); }
    let obj = this_val.as_obj();
    let len = vm.heap.array_len(obj);
    // Shift elements up
    let mut i = len;
    while i > 0 {
        let v = vm.heap.array_get(obj, i - 1);
        vm.heap.array_set(obj, i - 1 + argc as u32, v);
        i -= 1;
    }
    for i in 0..argc {
        vm.heap.array_set(obj, i as u32, vm.stack[args_start + i]);
    }
    vm.heap.set_array_len(obj, len + argc as u32);
    Ok(JsValue::from_i32((len + argc as u32) as i32))
}

pub fn array_index_of(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 || argc == 0 { return Ok(JsValue::from_i32(-1)); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_object() { return Ok(JsValue::from_i32(-1)); }
    let obj = this_val.as_obj();
    let search = vm.stack[args_start];
    let len = vm.heap.array_len(obj);
    for i in 0..len {
        let v = vm.heap.array_get(obj, i);
        if v.strict_eq(search) {
            return Ok(JsValue::from_i32(i as i32));
        }
    }
    Ok(JsValue::from_i32(-1))
}

pub fn array_last_index_of(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 || argc == 0 { return Ok(JsValue::from_i32(-1)); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_object() { return Ok(JsValue::from_i32(-1)); }
    let obj = this_val.as_obj();
    let search = vm.stack[args_start];
    let len = vm.heap.array_len(obj);
    let mut i = len;
    while i > 0 {
        i -= 1;
        let v = vm.heap.array_get(obj, i);
        if v.strict_eq(search) {
            return Ok(JsValue::from_i32(i as i32));
        }
    }
    Ok(JsValue::from_i32(-1))
}

pub fn array_includes(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 || argc == 0 { return Ok(JsValue::FALSE); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_object() { return Ok(JsValue::FALSE); }
    let obj = this_val.as_obj();
    let search = vm.stack[args_start];
    let len = vm.heap.array_len(obj);
    for i in 0..len {
        let v = vm.heap.array_get(obj, i);
        if v.strict_eq(search) { return Ok(JsValue::TRUE); }
    }
    Ok(JsValue::FALSE)
}

pub fn array_slice(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::UNDEFINED); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_object() { return Ok(JsValue::UNDEFINED); }
    let obj = this_val.as_obj();
    let len = vm.heap.array_len(obj) as i32;

    let mut start = if argc > 0 { vm.stack[args_start].to_i32() } else { 0 };
    let mut end = if argc > 1 { vm.stack[args_start + 1].to_i32() } else { len };
    if start < 0 { start += len; }
    if end < 0 { end += len; }
    if start < 0 { start = 0; }
    if end > len { end = len; }

    let new_len = if end > start { (end - start) as u32 } else { 0 };
    let result = vm.heap.alloc_array(new_len);
    for i in 0..new_len {
        let v = vm.heap.array_get(obj, (start as u32) + i);
        vm.heap.array_set(result, i, v);
    }
    Ok(JsValue::from_obj(result))
}

pub fn array_concat(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::UNDEFINED); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_object() { return Ok(JsValue::UNDEFINED); }
    let obj = this_val.as_obj();
    let len = vm.heap.array_len(obj);

    let result = vm.heap.alloc_array(len);
    // Copy this array
    for i in 0..len {
        let v = vm.heap.array_get(obj, i);
        vm.heap.array_set(result, i, v);
    }
    // Concat arguments
    let mut idx = len;
    for a in 0..argc {
        let arg = vm.stack[args_start + a];
        if arg.is_object() {
            let flags = vm.heap.get_flags(arg.as_obj());
            if flags & ObjFlags::ARRAY != 0 {
                let alen = vm.heap.array_len(arg.as_obj());
                for j in 0..alen {
                    let v = vm.heap.array_get(arg.as_obj(), j);
                    vm.heap.array_push(result, v);
                    idx += 1;
                }
                continue;
            }
        }
        vm.heap.array_push(result, arg);
        idx += 1;
    }
    Ok(JsValue::from_obj(result))
}

pub fn array_join(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::UNDEFINED); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_object() { return Ok(JsValue::UNDEFINED); }
    let obj = this_val.as_obj();
    let len = vm.heap.array_len(obj);

    let sep_sid = if argc > 0 && vm.stack[args_start].is_string() {
        vm.stack[args_start].as_str_id()
    } else {
        vm.strings.intern(b",")
    };

    let mut buf = [0u8; 4096];
    let mut pos = 0;
    for i in 0..len {
        if i > 0 {
            let sep = vm.strings.get(sep_sid);
            let n = sep.len().min(buf.len() - pos);
            buf[pos..pos + n].copy_from_slice(&sep[..n]);
            pos += n;
        }
        let v = vm.heap.array_get(obj, i);
        pos += v.write_to(&mut buf[pos..], &vm.strings);
    }
    let sid = vm.strings.intern(&buf[..pos]);
    Ok(JsValue::from_str(sid))
}

pub fn array_reverse(vm: &mut Vm, args_start: usize, _argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::UNDEFINED); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_object() { return Ok(JsValue::UNDEFINED); }
    let obj = this_val.as_obj();
    let len = vm.heap.array_len(obj);
    let mut i = 0;
    let mut j = if len > 0 { len - 1 } else { 0 };
    while i < j {
        let a = vm.heap.array_get(obj, i);
        let b = vm.heap.array_get(obj, j);
        vm.heap.array_set(obj, i, b);
        vm.heap.array_set(obj, j, a);
        i += 1;
        j -= 1;
    }
    Ok(this_val)
}

pub fn array_fill(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 || argc == 0 { return Ok(JsValue::UNDEFINED); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_object() { return Ok(JsValue::UNDEFINED); }
    let obj = this_val.as_obj();
    let len = vm.heap.array_len(obj) as i32;
    let fill_val = vm.stack[args_start];
    let start = if argc > 1 { vm.stack[args_start + 1].to_i32().max(0) } else { 0 };
    let end = if argc > 2 { vm.stack[args_start + 2].to_i32().min(len) } else { len };
    for i in start..end {
        vm.heap.array_set(obj, i as u32, fill_val);
    }
    Ok(this_val)
}

pub fn array_for_each(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 || argc == 0 { return Ok(JsValue::UNDEFINED); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_object() { return Ok(JsValue::UNDEFINED); }
    // forEach needs to call a callback — we'll just return undefined for now
    // Full implementation would require re-entering the VM loop
    Ok(JsValue::UNDEFINED)
}

pub fn array_every(_vm: &mut Vm, _args_start: usize, _argc: usize) -> Result<JsValue, JsError> {
    // Simplified — always returns true
    Ok(JsValue::TRUE)
}

pub fn array_some(_vm: &mut Vm, _args_start: usize, _argc: usize) -> Result<JsValue, JsError> {
    // Simplified — always returns false
    Ok(JsValue::FALSE)
}

pub fn array_find_index(_vm: &mut Vm, _args_start: usize, _argc: usize) -> Result<JsValue, JsError> {
    Ok(JsValue::from_i32(-1))
}

pub fn array_sort(vm: &mut Vm, args_start: usize, _argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::UNDEFINED); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_object() { return Ok(JsValue::UNDEFINED); }
    let obj = this_val.as_obj();
    let len = vm.heap.array_len(obj);
    // Simple bubble sort on numeric values
    if len <= 1 { return Ok(this_val); }
    for i in 0..len {
        for j in 0..len - 1 - i {
            let a = vm.heap.array_get(obj, j);
            let b = vm.heap.array_get(obj, j + 1);
            // Default sort: compare as strings or numbers
            let swap = if a.is_number() && b.is_number() {
                a.to_number() > b.to_number()
            } else {
                // Compare as strings
                let mut ba = [0u8; 64];
                let mut bb = [0u8; 64];
                let la = a.write_to(&mut ba, &vm.strings);
                let lb = b.write_to(&mut bb, &vm.strings);
                let cmp_len = la.min(lb);
                let mut cmp = 0i32;
                for k in 0..cmp_len {
                    if ba[k] != bb[k] {
                        cmp = ba[k] as i32 - bb[k] as i32;
                        break;
                    }
                }
                if cmp == 0 { la > lb } else { cmp > 0 }
            };
            if swap {
                vm.heap.array_set(obj, j, b);
                vm.heap.array_set(obj, j + 1, a);
            }
        }
    }
    Ok(this_val)
}

// ─── String prototype methods ───

pub fn string_char_at(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::from_str(StringId::EMPTY)); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_string() { return Ok(JsValue::from_str(StringId::EMPTY)); }
    let s = vm.strings.get(this_val.as_str_id());
    let idx = if argc > 0 { vm.stack[args_start].to_i32() as usize } else { 0 };
    if idx < s.len() {
        let ch_sid = vm.strings.intern(&[s[idx]]);
        Ok(JsValue::from_str(ch_sid))
    } else {
        Ok(JsValue::from_str(StringId::EMPTY))
    }
}

pub fn string_char_code_at(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::from_f64(f64::NAN)); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_string() { return Ok(JsValue::from_f64(f64::NAN)); }
    let s = vm.strings.get(this_val.as_str_id());
    let idx = if argc > 0 { vm.stack[args_start].to_i32() as usize } else { 0 };
    if idx < s.len() {
        Ok(JsValue::from_i32(s[idx] as i32))
    } else {
        Ok(JsValue::from_f64(f64::NAN))
    }
}

pub fn string_index_of(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 || argc == 0 { return Ok(JsValue::from_i32(-1)); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_string() { return Ok(JsValue::from_i32(-1)); }
    let s = vm.strings.get(this_val.as_str_id());
    let search_val = vm.stack[args_start];
    let search = if search_val.is_string() {
        vm.strings.get(search_val.as_str_id())
    } else {
        return Ok(JsValue::from_i32(-1));
    };
    if search.is_empty() { return Ok(JsValue::from_i32(0)); }

    // Copy to local buffers to avoid borrow issues
    let mut sbuf = [0u8; 1024];
    let slen = s.len().min(1024);
    sbuf[..slen].copy_from_slice(&s[..slen]);

    let mut nbuf = [0u8; 256];
    let nlen = search.len().min(256);
    nbuf[..nlen].copy_from_slice(&search[..nlen]);

    for i in 0..=slen.saturating_sub(nlen) {
        if sbuf[i..i + nlen] == nbuf[..nlen] {
            return Ok(JsValue::from_i32(i as i32));
        }
    }
    Ok(JsValue::from_i32(-1))
}

pub fn string_last_index_of(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 || argc == 0 { return Ok(JsValue::from_i32(-1)); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_string() { return Ok(JsValue::from_i32(-1)); }
    let s = vm.strings.get(this_val.as_str_id());
    let search_val = vm.stack[args_start];
    let search = if search_val.is_string() {
        vm.strings.get(search_val.as_str_id())
    } else {
        return Ok(JsValue::from_i32(-1));
    };

    let mut sbuf = [0u8; 1024];
    let slen = s.len().min(1024);
    sbuf[..slen].copy_from_slice(&s[..slen]);
    let mut nbuf = [0u8; 256];
    let nlen = search.len().min(256);
    nbuf[..nlen].copy_from_slice(&search[..nlen]);

    if nlen == 0 { return Ok(JsValue::from_i32(slen as i32)); }
    let mut i = slen.saturating_sub(nlen);
    loop {
        if sbuf[i..i + nlen] == nbuf[..nlen] {
            return Ok(JsValue::from_i32(i as i32));
        }
        if i == 0 { break; }
        i -= 1;
    }
    Ok(JsValue::from_i32(-1))
}

pub fn string_includes(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    let result = string_index_of(vm, args_start, argc)?;
    Ok(JsValue::from_bool(result.to_i32() >= 0))
}

pub fn string_starts_with(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 || argc == 0 { return Ok(JsValue::FALSE); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_string() { return Ok(JsValue::FALSE); }
    let s = vm.strings.get(this_val.as_str_id());
    let prefix_val = vm.stack[args_start];
    if !prefix_val.is_string() { return Ok(JsValue::FALSE); }
    let prefix = vm.strings.get(prefix_val.as_str_id());
    if prefix.len() > s.len() { return Ok(JsValue::FALSE); }

    let mut sbuf = [0u8; 256];
    let plen = prefix.len().min(256);
    sbuf[..plen].copy_from_slice(&prefix[..plen]);

    let mut tbuf = [0u8; 256];
    tbuf[..plen].copy_from_slice(&s[..plen]);

    Ok(JsValue::from_bool(sbuf[..plen] == tbuf[..plen]))
}

pub fn string_ends_with(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 || argc == 0 { return Ok(JsValue::FALSE); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_string() { return Ok(JsValue::FALSE); }
    let s = vm.strings.get(this_val.as_str_id());
    let suffix_val = vm.stack[args_start];
    if !suffix_val.is_string() { return Ok(JsValue::FALSE); }
    let suffix = vm.strings.get(suffix_val.as_str_id());
    if suffix.len() > s.len() { return Ok(JsValue::FALSE); }
    let start = s.len() - suffix.len();

    let mut sbuf = [0u8; 256];
    let flen = suffix.len().min(256);
    sbuf[..flen].copy_from_slice(&suffix[..flen]);

    let mut tbuf = [0u8; 256];
    tbuf[..flen].copy_from_slice(&s[start..start + flen]);

    Ok(JsValue::from_bool(sbuf[..flen] == tbuf[..flen]))
}

pub fn string_slice(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::from_str(StringId::EMPTY)); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_string() { return Ok(JsValue::from_str(StringId::EMPTY)); }
    let s = vm.strings.get(this_val.as_str_id());
    let slen = s.len() as i32;

    let mut start = if argc > 0 { vm.stack[args_start].to_i32() } else { 0 };
    let mut end = if argc > 1 { vm.stack[args_start + 1].to_i32() } else { slen };
    if start < 0 { start += slen; }
    if end < 0 { end += slen; }
    if start < 0 { start = 0; }
    if end > slen { end = slen; }
    if start >= end { return Ok(JsValue::from_str(StringId::EMPTY)); }

    let mut buf = [0u8; 1024];
    let len = (end - start) as usize;
    let clen = len.min(1024);
    buf[..clen].copy_from_slice(&s[start as usize..start as usize + clen]);
    let sid = vm.strings.intern(&buf[..clen]);
    Ok(JsValue::from_str(sid))
}

pub fn string_substring(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    string_slice(vm, args_start, argc)
}

pub fn string_split(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::UNDEFINED); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_string() { return Ok(JsValue::UNDEFINED); }
    let s = vm.strings.get(this_val.as_str_id());

    let result = vm.heap.alloc_array(0);

    if argc == 0 {
        // No separator — return array with entire string
        vm.heap.array_push(result, this_val);
        return Ok(JsValue::from_obj(result));
    }

    let sep_val = vm.stack[args_start];
    if !sep_val.is_string() {
        vm.heap.array_push(result, this_val);
        return Ok(JsValue::from_obj(result));
    }
    let sep = vm.strings.get(sep_val.as_str_id());

    // Copy to local buffers
    let mut sbuf = [0u8; 1024];
    let slen = s.len().min(1024);
    sbuf[..slen].copy_from_slice(&s[..slen]);

    let mut sepbuf = [0u8; 128];
    let seplen = sep.len().min(128);
    sepbuf[..seplen].copy_from_slice(&sep[..seplen]);

    if seplen == 0 {
        // Split each character
        for i in 0..slen {
            let ch_sid = vm.strings.intern(&sbuf[i..i + 1]);
            vm.heap.array_push(result, JsValue::from_str(ch_sid));
        }
        return Ok(JsValue::from_obj(result));
    }

    let mut start = 0;
    while start <= slen {
        let mut found = false;
        let mut end = start;
        while end + seplen <= slen {
            if sbuf[end..end + seplen] == sepbuf[..seplen] {
                let sid = vm.strings.intern(&sbuf[start..end]);
                vm.heap.array_push(result, JsValue::from_str(sid));
                start = end + seplen;
                found = true;
                break;
            }
            end += 1;
        }
        if !found {
            let sid = vm.strings.intern(&sbuf[start..slen]);
            vm.heap.array_push(result, JsValue::from_str(sid));
            break;
        }
    }

    Ok(JsValue::from_obj(result))
}

pub fn string_replace(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 || argc < 2 { return Ok(JsValue::UNDEFINED); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_string() { return Ok(JsValue::UNDEFINED); }
    let s = vm.strings.get(this_val.as_str_id());

    let search_val = vm.stack[args_start];
    let replace_val = vm.stack[args_start + 1];
    if !search_val.is_string() || !replace_val.is_string() { return Ok(this_val); }
    let search = vm.strings.get(search_val.as_str_id());
    let replace = vm.strings.get(replace_val.as_str_id());

    let mut sbuf = [0u8; 1024];
    let slen = s.len().min(1024);
    sbuf[..slen].copy_from_slice(&s[..slen]);

    let mut nbuf = [0u8; 256];
    let nlen = search.len().min(256);
    nbuf[..nlen].copy_from_slice(&search[..nlen]);

    let mut rbuf = [0u8; 256];
    let rlen = replace.len().min(256);
    rbuf[..rlen].copy_from_slice(&replace[..rlen]);

    // Find first occurrence and replace
    let mut result = [0u8; 2048];
    let mut rpos = 0;
    let mut i = 0;
    let mut replaced = false;
    while i < slen {
        if !replaced && i + nlen <= slen && sbuf[i..i + nlen] == nbuf[..nlen] {
            result[rpos..rpos + rlen].copy_from_slice(&rbuf[..rlen]);
            rpos += rlen;
            i += nlen;
            replaced = true;
        } else {
            result[rpos] = sbuf[i];
            rpos += 1;
            i += 1;
        }
    }
    let sid = vm.strings.intern(&result[..rpos]);
    Ok(JsValue::from_str(sid))
}

pub fn string_replace_all(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 || argc < 2 { return Ok(JsValue::UNDEFINED); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_string() { return Ok(JsValue::UNDEFINED); }
    let s = vm.strings.get(this_val.as_str_id());
    let search_val = vm.stack[args_start];
    let replace_val = vm.stack[args_start + 1];
    if !search_val.is_string() || !replace_val.is_string() { return Ok(this_val); }
    let search = vm.strings.get(search_val.as_str_id());
    let replace = vm.strings.get(replace_val.as_str_id());

    let mut sbuf = [0u8; 1024];
    let slen = s.len().min(1024);
    sbuf[..slen].copy_from_slice(&s[..slen]);
    let mut nbuf = [0u8; 256];
    let nlen = search.len().min(256);
    nbuf[..nlen].copy_from_slice(&search[..nlen]);
    let mut rbuf = [0u8; 256];
    let rlen = replace.len().min(256);
    rbuf[..rlen].copy_from_slice(&replace[..rlen]);

    let mut result = [0u8; 2048];
    let mut rpos = 0;
    let mut i = 0;
    while i < slen {
        if nlen > 0 && i + nlen <= slen && sbuf[i..i + nlen] == nbuf[..nlen] {
            result[rpos..rpos + rlen].copy_from_slice(&rbuf[..rlen]);
            rpos += rlen;
            i += nlen;
        } else {
            result[rpos] = sbuf[i];
            rpos += 1;
            i += 1;
        }
    }
    let sid = vm.strings.intern(&result[..rpos]);
    Ok(JsValue::from_str(sid))
}

pub fn string_trim(vm: &mut Vm, args_start: usize, _argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::from_str(StringId::EMPTY)); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_string() { return Ok(JsValue::from_str(StringId::EMPTY)); }
    let mut buf = [0u8; 1024];
    let slen;
    {
        let s = vm.strings.get(this_val.as_str_id());
        slen = s.len().min(1024);
        buf[..slen].copy_from_slice(&s[..slen]);
    }
    let mut start = 0;
    let mut end = slen;
    while start < end && is_ws(buf[start]) { start += 1; }
    while end > start && is_ws(buf[end - 1]) { end -= 1; }
    let sid = vm.strings.intern(&buf[start..end]);
    Ok(JsValue::from_str(sid))
}

pub fn string_trim_start(vm: &mut Vm, args_start: usize, _argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::from_str(StringId::EMPTY)); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_string() { return Ok(JsValue::from_str(StringId::EMPTY)); }
    let mut buf = [0u8; 1024];
    let slen;
    {
        let s = vm.strings.get(this_val.as_str_id());
        slen = s.len().min(1024);
        buf[..slen].copy_from_slice(&s[..slen]);
    }
    let mut start = 0;
    while start < slen && is_ws(buf[start]) { start += 1; }
    let sid = vm.strings.intern(&buf[start..slen]);
    Ok(JsValue::from_str(sid))
}

pub fn string_trim_end(vm: &mut Vm, args_start: usize, _argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::from_str(StringId::EMPTY)); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_string() { return Ok(JsValue::from_str(StringId::EMPTY)); }
    let mut buf = [0u8; 1024];
    let slen;
    {
        let s = vm.strings.get(this_val.as_str_id());
        slen = s.len().min(1024);
        buf[..slen].copy_from_slice(&s[..slen]);
    }
    let mut end = slen;
    while end > 0 && is_ws(buf[end - 1]) { end -= 1; }
    let sid = vm.strings.intern(&buf[..end]);
    Ok(JsValue::from_str(sid))
}

pub fn string_to_lower(vm: &mut Vm, args_start: usize, _argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::from_str(StringId::EMPTY)); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_string() { return Ok(JsValue::from_str(StringId::EMPTY)); }
    let s = vm.strings.get(this_val.as_str_id());
    let mut buf = [0u8; 1024];
    let len = s.len().min(1024);
    for i in 0..len {
        buf[i] = if s[i] >= b'A' && s[i] <= b'Z' { s[i] + 32 } else { s[i] };
    }
    let sid = vm.strings.intern(&buf[..len]);
    Ok(JsValue::from_str(sid))
}

pub fn string_to_upper(vm: &mut Vm, args_start: usize, _argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::from_str(StringId::EMPTY)); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_string() { return Ok(JsValue::from_str(StringId::EMPTY)); }
    let s = vm.strings.get(this_val.as_str_id());
    let mut buf = [0u8; 1024];
    let len = s.len().min(1024);
    for i in 0..len {
        buf[i] = if s[i] >= b'a' && s[i] <= b'z' { s[i] - 32 } else { s[i] };
    }
    let sid = vm.strings.intern(&buf[..len]);
    Ok(JsValue::from_str(sid))
}

pub fn string_repeat(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::from_str(StringId::EMPTY)); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_string() { return Ok(JsValue::from_str(StringId::EMPTY)); }
    let s = vm.strings.get(this_val.as_str_id());
    let count = if argc > 0 { vm.stack[args_start].to_i32().max(0) as usize } else { 0 };
    let slen = s.len();
    let _total = (slen * count).min(4096);
    let mut buf = [0u8; 4096];

    // Copy source to local buffer first
    let mut src = [0u8; 256];
    let src_len = slen.min(256);
    src[..src_len].copy_from_slice(&s[..src_len]);

    let mut pos = 0;
    for _ in 0..count {
        if pos + src_len > 4096 { break; }
        buf[pos..pos + src_len].copy_from_slice(&src[..src_len]);
        pos += src_len;
    }
    let sid = vm.strings.intern(&buf[..pos]);
    Ok(JsValue::from_str(sid))
}

pub fn string_pad_start(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 || argc == 0 { return Ok(JsValue::UNDEFINED); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_string() { return Ok(this_val); }
    let s = vm.strings.get(this_val.as_str_id());
    let target_len = vm.stack[args_start].to_i32().max(0) as usize;
    if s.len() >= target_len { return Ok(this_val); }

    let pad_str = if argc > 1 && vm.stack[args_start + 1].is_string() {
        vm.strings.get(vm.stack[args_start + 1].as_str_id())
    } else {
        b" " as &[u8]
    };

    let pad_needed = target_len - s.len();
    let mut buf = [0u8; 1024];
    let mut pos = 0;

    // Copy pad string to local buffer
    let mut pbuf = [0u8; 128];
    let plen = pad_str.len().min(128);
    pbuf[..plen].copy_from_slice(&pad_str[..plen]);

    // Fill padding
    while pos < pad_needed && pos < 1024 {
        buf[pos] = pbuf[pos % plen];
        pos += 1;
    }
    // Copy original
    let slen = s.len().min(1024 - pos);
    // Must re-get because we may have invalidated
    let s2 = vm.strings.get(this_val.as_str_id());
    buf[pos..pos + slen].copy_from_slice(&s2[..slen]);
    pos += slen;
    let sid = vm.strings.intern(&buf[..pos]);
    Ok(JsValue::from_str(sid))
}

pub fn string_pad_end(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 || argc == 0 { return Ok(JsValue::UNDEFINED); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_string() { return Ok(this_val); }
    let s = vm.strings.get(this_val.as_str_id());
    let target_len = vm.stack[args_start].to_i32().max(0) as usize;
    if s.len() >= target_len { return Ok(this_val); }

    let pad_str = if argc > 1 && vm.stack[args_start + 1].is_string() {
        vm.strings.get(vm.stack[args_start + 1].as_str_id())
    } else {
        b" " as &[u8]
    };

    let pad_needed = target_len - s.len();
    let mut buf = [0u8; 1024];
    let slen = s.len().min(1024);
    buf[..slen].copy_from_slice(&s[..slen]);
    let mut pos = slen;

    let mut pbuf = [0u8; 128];
    let plen = pad_str.len().min(128);
    pbuf[..plen].copy_from_slice(&pad_str[..plen]);

    let mut pi = 0;
    while pi < pad_needed && pos < 1024 {
        buf[pos] = pbuf[pi % plen];
        pos += 1;
        pi += 1;
    }
    let sid = vm.strings.intern(&buf[..pos]);
    Ok(JsValue::from_str(sid))
}

pub fn string_concat(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::from_str(StringId::EMPTY)); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_string() { return Ok(this_val); }
    let mut result_sid = this_val.as_str_id();
    for i in 0..argc {
        let arg = vm.stack[args_start + i];
        let arg_sid = if arg.is_string() {
            arg.as_str_id()
        } else {
            let mut buf = [0u8; 64];
            let n = arg.write_to(&mut buf, &vm.strings);
            vm.strings.intern(&buf[..n])
        };
        result_sid = vm.strings.concat(result_sid, arg_sid);
    }
    Ok(JsValue::from_str(result_sid))
}

pub fn string_at(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::UNDEFINED); }
    let this_val = vm.stack[args_start - 1];
    if !this_val.is_string() { return Ok(JsValue::UNDEFINED); }
    let s = vm.strings.get(this_val.as_str_id());
    let mut idx = if argc > 0 { vm.stack[args_start].to_i32() } else { 0 };
    if idx < 0 { idx += s.len() as i32; }
    if idx < 0 || idx as usize >= s.len() { return Ok(JsValue::UNDEFINED); }
    let ch_sid = vm.strings.intern(&[s[idx as usize]]);
    Ok(JsValue::from_str(ch_sid))
}

// ─── Number methods ───

pub fn number_to_fixed(vm: &mut Vm, args_start: usize, argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::from_str(StringId::EMPTY)); }
    let this_val = vm.stack[args_start - 1];
    let v = this_val.to_number();
    let digits = if argc > 0 { vm.stack[args_start].to_i32().max(0).min(20) as usize } else { 0 };

    let mut buf = [0u8; 64];
    let n = write_f64_fixed(&mut buf, v, digits);
    let sid = vm.strings.intern(&buf[..n]);
    Ok(JsValue::from_str(sid))
}

pub fn number_to_string_method(vm: &mut Vm, args_start: usize, _argc: usize) -> Result<JsValue, JsError> {
    if args_start == 0 { return Ok(JsValue::from_str(StringId::EMPTY)); }
    let this_val = vm.stack[args_start - 1];
    let mut buf = [0u8; 64];
    let n = this_val.write_to(&mut buf, &vm.strings);
    let sid = vm.strings.intern(&buf[..n]);
    Ok(JsValue::from_str(sid))
}

// ─── Helpers ───

fn is_ws(ch: u8) -> bool {
    ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r'
}

fn write_f64_fixed(buf: &mut [u8], v: f64, digits: usize) -> usize {
    if buf.len() < 32 { return 0; }
    let mut pos = 0;
    let neg = v < 0.0;
    let v = if neg { -v } else { v };
    if neg { buf[pos] = b'-'; pos += 1; }

    // Multiply by 10^digits and round
    let mut mult = 1.0;
    for _ in 0..digits { mult *= 10.0; }
    let rounded = (v * mult + 0.5) as u64;
    let int_part = rounded / mult as u64;
    let frac_part = rounded % mult as u64;

    // Write integer part
    let mut tmp = [0u8; 20];
    let mut ti = 0;
    let mut n = int_part;
    if n == 0 {
        tmp[0] = b'0';
        ti = 1;
    } else {
        while n > 0 {
            tmp[ti] = b'0' + (n % 10) as u8;
            n /= 10;
            ti += 1;
        }
    }
    for i in (0..ti).rev() {
        buf[pos] = tmp[i];
        pos += 1;
    }

    if digits > 0 {
        buf[pos] = b'.';
        pos += 1;
        // Write fractional part with leading zeros
        let mut f = frac_part;
        let mut fdigits = [0u8; 20];
        for i in (0..digits).rev() {
            fdigits[i] = b'0' + (f % 10) as u8;
            f /= 10;
        }
        for i in 0..digits {
            buf[pos] = fdigits[i];
            pos += 1;
        }
    }
    pos
}

/// Register all builtin methods on the VM. Called during VM init.
/// Creates Array.prototype and String.prototype with native methods,
/// then stores them on the VM for use when creating new arrays/strings.
pub fn register_builtins(vm: &mut Vm) {
    // ─── Array.prototype ───
    let array_proto = vm.heap.alloc_object();
    add_method(vm, array_proto, b"push", array_push);
    add_method(vm, array_proto, b"pop", array_pop);
    add_method(vm, array_proto, b"shift", array_shift);
    add_method(vm, array_proto, b"unshift", array_unshift);
    add_method(vm, array_proto, b"indexOf", array_index_of);
    add_method(vm, array_proto, b"lastIndexOf", array_last_index_of);
    add_method(vm, array_proto, b"includes", array_includes);
    add_method(vm, array_proto, b"slice", array_slice);
    add_method(vm, array_proto, b"concat", array_concat);
    add_method(vm, array_proto, b"join", array_join);
    add_method(vm, array_proto, b"reverse", array_reverse);
    add_method(vm, array_proto, b"sort", array_sort);
    add_method(vm, array_proto, b"fill", array_fill);
    add_method(vm, array_proto, b"findIndex", array_find_index);
    add_method(vm, array_proto, b"forEach", array_for_each);
    add_method(vm, array_proto, b"every", array_every);
    add_method(vm, array_proto, b"some", array_some);
    vm.array_proto = array_proto;

    // ─── String.prototype ───
    let string_proto = vm.heap.alloc_object();
    add_method(vm, string_proto, b"charAt", string_char_at);
    add_method(vm, string_proto, b"charCodeAt", string_char_code_at);
    add_method(vm, string_proto, b"indexOf", string_index_of);
    add_method(vm, string_proto, b"lastIndexOf", string_last_index_of);
    add_method(vm, string_proto, b"includes", string_includes);
    add_method(vm, string_proto, b"startsWith", string_starts_with);
    add_method(vm, string_proto, b"endsWith", string_ends_with);
    add_method(vm, string_proto, b"slice", string_slice);
    add_method(vm, string_proto, b"substring", string_substring);
    add_method(vm, string_proto, b"split", string_split);
    add_method(vm, string_proto, b"replace", string_replace);
    add_method(vm, string_proto, b"replaceAll", string_replace_all);
    add_method(vm, string_proto, b"trim", string_trim);
    add_method(vm, string_proto, b"trimStart", string_trim_start);
    add_method(vm, string_proto, b"trimEnd", string_trim_end);
    add_method(vm, string_proto, b"toLowerCase", string_to_lower);
    add_method(vm, string_proto, b"toUpperCase", string_to_upper);
    add_method(vm, string_proto, b"repeat", string_repeat);
    add_method(vm, string_proto, b"padStart", string_pad_start);
    add_method(vm, string_proto, b"padEnd", string_pad_end);
    add_method(vm, string_proto, b"concat", string_concat);
    add_method(vm, string_proto, b"at", string_at);
    vm.string_proto = string_proto;

    // ─── Number.prototype ───
    let number_proto = vm.heap.alloc_object();
    add_method(vm, number_proto, b"toFixed", number_to_fixed);
    add_method(vm, number_proto, b"toString", number_to_string_method);
    vm.number_proto = number_proto;
}

/// Helper: create a native function and attach it as a property on an object.
fn add_method(
    vm: &mut Vm,
    obj: ObjId,
    name: &[u8],
    func: fn(&mut Vm, usize, usize) -> Result<JsValue, JsError>,
) {
    let func_obj = vm.make_native_function(func);
    let name_id = vm.strings.intern(name);
    vm.heap.set_prop(obj, name_id, JsValue::from_obj(func_obj));
}
