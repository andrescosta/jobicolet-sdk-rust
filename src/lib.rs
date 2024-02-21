extern crate alloc;
extern crate core;
extern crate wee_alloc;
use alloc::vec::Vec;
use std::mem::MaybeUninit;
use std::slice;

#[repr(C)]
pub struct Ret2Val {
    one: u64,
    two: u64,
}

pub type EventFunc = fn(&String) -> (u64, String);

pub static mut ON_EVENT: Option<EventFunc> = None;

pub fn log(level: u32, message: &String) {
    unsafe {
        let (ptr, len) = string_to_ptr(message);
        _log(level, ptr, len);
    }
}

#[link(wasm_import_module = "env")]
extern "C" {
    #[link_name = "log"]
    fn _log(level: u32, ptr: u32, size: u32);
}

#[cfg_attr(all(target_arch = "wasm32"), export_name = "event")]
#[no_mangle]
unsafe extern "C" fn _event(ptr: u32, len: u32) -> Ret2Val {
    let name = &ptr_to_string(ptr, len);
    let (errno, result) = ON_EVENT.unwrap()(name);
    let (ptr, len) = string_to_ptr(&result);
    std::mem::forget(result);
    return Ret2Val {
        one: errno,
        two: ((ptr as u64) << 32) | len as u64,
    };
}

unsafe fn ptr_to_string(ptr: u32, len: u32) -> String {
    let slice = slice::from_raw_parts_mut(ptr as *mut u8, len as usize);
    let utf8 = std::str::from_utf8_unchecked_mut(slice);
    return String::from(utf8);
}

unsafe fn string_to_ptr(s: &String) -> (u32, u32) {
    return (s.as_ptr() as u32, s.len() as u32);
}

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg_attr(all(target_arch = "wasm32"), export_name = "malloc")]
#[no_mangle]
extern "C" fn _malloc(size: u32) -> *mut u8 {
    malloc(size as usize)
}

#[cfg_attr(all(target_arch = "wasm32"), export_name = "ver")]
#[no_mangle]
extern "C" fn _ver() -> u8 {
    return 1;
}


fn malloc(size: usize) -> *mut u8 {
    let vec: Vec<MaybeUninit<u8>> = Vec::with_capacity(size);
    Box::into_raw(vec.into_boxed_slice()) as *mut u8
}

#[cfg_attr(all(target_arch = "wasm32"), export_name = "free")]
#[no_mangle]
unsafe extern "C" fn _free(ptr: u32, size: u32) {
    free(ptr as *mut u8, size as usize);
}

unsafe fn free(ptr: *mut u8, size: usize) {
    let _ = Vec::from_raw_parts(ptr, 0, size);
}
