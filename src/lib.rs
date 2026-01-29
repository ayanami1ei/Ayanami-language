use macro_lib::{logic_operation, type_trans, value_operation};
use std::{
    alloc::{alloc, dealloc, Layout},
    ptr::{self, null_mut},
};

//cargo clean && cargo build --lib --release

use crate::types::{BoolObject, CharObject, FloatObject, IntObject, Object, StringObject, VarType};
mod types;

#[unsafe(no_mangle)]
pub extern "C" fn err(obj: *mut Object) {
    write(obj);
    std::process::abort();
}

#[unsafe(no_mangle)]
pub extern "C" fn write(obj: *mut Object) {
    unsafe {
        let s = &*(obj as *mut StringObject);
        let slice = std::slice::from_raw_parts(s.data, s.len);
        let string = std::str::from_utf8(slice).unwrap();
        println!("{}", string);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn is_type(ty: u32, obj: *mut Object) -> bool {
    unsafe { (*obj).ty.clone() as u32 == ty }
}

pub extern "C" fn del_obj_inner<T>(obj: *mut Object) {
    let layout = Layout::new::<T>();
    unsafe { dealloc(obj as *mut u8, layout) };
}

#[unsafe(no_mangle)]
pub extern "C" fn del_obj(obj: *mut Object) {
    unsafe {
        // 释放对象本身
        match (*obj).ty {
            VarType::Int => del_obj_inner::<IntObject>(obj),
            VarType::Float => del_obj_inner::<FloatObject>(obj),
            VarType::Bool => del_obj_inner::<BoolObject>(obj),
            VarType::Char => del_obj_inner::<CharObject>(obj),
            VarType::String => del_obj_inner::<CharObject>(obj),
            VarType::Unknown => panic!("no such type"),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dec_ref(obj: *mut Object) {
    if obj.is_null() {
        return;
    }

    unsafe {
        // 先减引用计数
        (*obj).refcnt -= 1;

        // 如果没有引用了，就释放
        if (*obj).refcnt == 0 {
            del_obj(obj);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn inc_ref(obj: *mut Object) {
    if obj.is_null() {
        return;
    }

    unsafe {
        (*obj).refcnt += 1;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn alloc_int(value: i64) -> *mut Object {
    let layout = Layout::new::<IntObject>();
    unsafe {
        let ptr = alloc(layout) as *mut IntObject;

        if ptr.is_null() {
            return null_mut();
        }

        (*ptr).header.ty = VarType::Int;
        (*ptr).header.refcnt = 1;
        (*ptr).value = value;

        ptr as *mut Object
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn alloc_float(value: f64) -> *mut Object {
    let layout = Layout::new::<IntObject>();
    unsafe {
        let ptr = alloc(layout) as *mut FloatObject;

        if ptr.is_null() {
            return null_mut();
        }

        (*ptr).header.ty = VarType::Float;
        (*ptr).header.refcnt = 1;
        (*ptr).value = value;

        ptr as *mut Object
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn alloc_bool(value: bool) -> *mut Object {
    let layout = Layout::new::<IntObject>();
    unsafe {
        let ptr = alloc(layout) as *mut BoolObject;

        if ptr.is_null() {
            return null_mut();
        }

        (*ptr).header.ty = VarType::Bool;
        (*ptr).header.refcnt = 1;
        (*ptr).value = value;

        ptr as *mut Object
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn alloc_char(value: u8) -> *mut Object {
    let layout = Layout::new::<IntObject>();
    unsafe {
        let ptr = alloc(layout) as *mut CharObject;

        if ptr.is_null() {
            return null_mut();
        }

        (*ptr).header.ty = VarType::Char;
        (*ptr).header.refcnt = 1;
        (*ptr).value = value;

        ptr as *mut Object
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn alloc_string(value: *const u8, len: usize) -> *mut Object {
    let layout = Layout::new::<IntObject>();
    unsafe {
        let ptr = alloc(layout) as *mut StringObject;

        if ptr.is_null() {
            return null_mut();
        }

        (*ptr).header.ty = VarType::String;
        (*ptr).header.refcnt = 1;
        (*ptr).len = len;
        (*ptr).data = value;

        ptr as *mut Object
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn add(_left: *const Object, _right: *const Object) -> *mut Object {
    value_operation!(_left, +, _right)
}

#[unsafe(no_mangle)]
pub extern "C" fn sub(_left: *const Object, _right: *const Object) -> *mut Object {
    value_operation!(_left, -, _right)
}

#[unsafe(no_mangle)]
pub extern "C" fn mul(_left: *const Object, _right: *const Object) -> *mut Object {
    value_operation!(_left, *, _right)
}

#[unsafe(no_mangle)]
pub extern "C" fn div(_left: *const Object, _right: *const Object) -> *mut Object {
    value_operation!(_left, /, _right)
}

#[unsafe(no_mangle)]
pub extern "C" fn and(_left: *const Object, _right: *const Object) -> *mut Object {
    logic_operation!(_left, &&, _right)
}

#[unsafe(no_mangle)]
pub extern "C" fn or(_left: *const Object, _right: *const Object) -> *mut Object {
    logic_operation!(_left, ||, _right)
}

#[unsafe(no_mangle)]
pub extern "C" fn not(_cond: *const Object) -> *mut Object {
    unsafe {
        let cond = type_trans!(_cond, BoolObject);
        let v = (*cond).value;
        alloc_bool(!v)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn equal(_left: *const Object, _right: *const Object) -> *mut Object {
    logic_operation!(_left, ==, _right)
}

#[unsafe(no_mangle)]
pub extern "C" fn greater(_left: *const Object, _right: *const Object) -> *mut Object {
    logic_operation!(_left, >, _right)
}

#[unsafe(no_mangle)]
pub extern "C" fn less(_left: *const Object, _right: *const Object) -> *mut Object {
    logic_operation!(_left, <, _right)
}

#[unsafe(no_mangle)]
pub extern "C" fn greater_equal(_left: *const Object, _right: *const Object) -> *mut Object {
    logic_operation!(_left, >=, _right)
}

#[unsafe(no_mangle)]
pub extern "C" fn less_equal(_left: *const Object, _right: *const Object) -> *mut Object {
    logic_operation!(_left, <=, _right)
}
