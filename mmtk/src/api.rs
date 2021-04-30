use libc::c_char;
use libc::c_void;
use mmtk::memory_manager;
use mmtk::scheduler::GCWorker;
use mmtk::util::{Address, ObjectReference, OpaquePointer};
use mmtk::AllocationSemantics;
use mmtk::Mutator;
use mmtk::MMTK;
use mmtk::policy::space::Space;
use std::ffi::CStr;

use V8_Upcalls;
use UPCALLS;
use V8;

#[no_mangle]
pub unsafe extern "C" fn release_buffer(ptr: *mut Address, length: usize, capacity: usize) {
    let _vec = Vec::<Address>::from_raw_parts(ptr, length, capacity);
}

#[no_mangle]
pub extern "C" fn v8_new_heap(calls: *const V8_Upcalls, heap_size: usize) -> *mut c_void {
    unsafe {
        UPCALLS = calls;
    };
    let mmtk: *const MMTK<V8> = &*crate::SINGLETON;
    memory_manager::gc_init(unsafe { &mut *(mmtk as *mut MMTK<V8>) }, heap_size);
    enable_collection(unsafe { &mut *(mmtk as *mut MMTK<V8>) }, OpaquePointer::UNINITIALIZED);

    mmtk as *mut c_void
}

#[no_mangle]
pub extern "C" fn start_control_collector(mmtk: &mut MMTK<V8>, tls: OpaquePointer) {
    memory_manager::start_control_collector(&*mmtk, tls);
}

#[no_mangle]
pub extern "C" fn bind_mutator(
    mmtk: &'static mut MMTK<V8>,
    tls: OpaquePointer,
) -> *mut Mutator<V8> {
    Box::into_raw(memory_manager::bind_mutator(mmtk, tls))
}

#[no_mangle]
pub unsafe extern "C" fn mmtk_in_space(mmtk: &'static MMTK<V8>, object: ObjectReference, space: AllocationSemantics) -> i32 {
    let unsync = &*mmtk.plan.base().unsync.get();
    match space {
        AllocationSemantics::ReadOnly => unsync.ro_space.in_space(object) as _,
        _ => unreachable!()
    }
}

#[no_mangle]
// It is fine we turn the pointer back to box, as we turned a boxed value to the raw pointer in bind_mutator()
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn destroy_mutator(mutator: *mut Mutator<V8>) {
    memory_manager::destroy_mutator(unsafe { Box::from_raw(mutator) })
}

#[no_mangle]
pub extern "C" fn alloc(
    mutator: &mut Mutator<V8>,
    size: usize,
    align: usize,
    offset: isize,
    semantics: AllocationSemantics,
) -> Address {
    let a = memory_manager::alloc::<V8>(mutator, size, align, offset, semantics);
    unsafe { memory_manager::post_alloc::<V8>(mutator, a.to_object_reference(), size, semantics); }
    a
}

#[no_mangle]
pub extern "C" fn post_alloc(
    mutator: &mut Mutator<V8>,
    refer: ObjectReference,
    bytes: usize,
    semantics: AllocationSemantics,
) {
    memory_manager::post_alloc::<V8>(mutator, refer, bytes, semantics)
}

#[no_mangle]
pub extern "C" fn will_never_move(object: ObjectReference) -> bool {
    !object.is_movable()
}

#[no_mangle]
pub extern "C" fn start_worker(
    mmtk: &'static mut MMTK<V8>,
    tls: OpaquePointer,
    worker: &'static mut GCWorker<V8>,
) {
    memory_manager::start_worker::<V8>(tls, worker, mmtk);
}

#[no_mangle]
pub extern "C" fn enable_collection(mmtk: &'static mut MMTK<V8>, tls: OpaquePointer) {
    memory_manager::enable_collection(mmtk, tls);
}

#[no_mangle]
pub extern "C" fn used_bytes(mmtk: &mut MMTK<V8>) -> usize {
    memory_manager::used_bytes(mmtk)
}

#[no_mangle]
pub extern "C" fn free_bytes(mmtk: &mut MMTK<V8>) -> usize {
    memory_manager::free_bytes(mmtk)
}

#[no_mangle]
pub extern "C" fn total_bytes(mmtk: &mut MMTK<V8>) -> usize {
    memory_manager::total_bytes(&*mmtk)
}

#[no_mangle]
#[cfg(feature = "sanity")]
pub extern "C" fn scan_region(mmtk: &mut MMTK<V8>) {
    memory_manager::scan_region(mmtk);
}

#[no_mangle]
pub extern "C" fn is_live_object(object: ObjectReference) -> bool {
    object.is_live()
}

#[no_mangle]
pub extern "C" fn is_mapped_object(object: ObjectReference) -> bool {
    object.is_mapped()
}

#[no_mangle]
pub extern "C" fn is_mapped_address(address: Address) -> bool {
    address.is_mapped()
}

#[no_mangle]
pub extern "C" fn modify_check(mmtk: &mut MMTK<V8>, object: ObjectReference) {
    memory_manager::modify_check(mmtk, object);
}

#[no_mangle]
pub extern "C" fn handle_user_collection_request(mmtk: &mut MMTK<V8>, tls: OpaquePointer) {
    memory_manager::handle_user_collection_request::<V8>(mmtk, tls);
}

#[no_mangle]
pub extern "C" fn add_weak_candidate(
    mmtk: &mut MMTK<V8>,
    reff: ObjectReference,
    referent: ObjectReference,
) {
    memory_manager::add_weak_candidate(mmtk, reff, referent);
}

#[no_mangle]
pub extern "C" fn add_soft_candidate(
    mmtk: &mut MMTK<V8>,
    reff: ObjectReference,
    referent: ObjectReference,
) {
    memory_manager::add_soft_candidate(mmtk, reff, referent);
}

#[no_mangle]
pub extern "C" fn add_phantom_candidate(
    mmtk: &mut MMTK<V8>,
    reff: ObjectReference,
    referent: ObjectReference,
) {
    memory_manager::add_phantom_candidate(mmtk, reff, referent);
}

#[no_mangle]
pub extern "C" fn harness_begin(mmtk: &mut MMTK<V8>, tls: OpaquePointer) {
    memory_manager::harness_begin(mmtk, tls);
}

#[no_mangle]
pub extern "C" fn harness_end(mmtk: &'static mut MMTK<V8>, _tls: OpaquePointer) {
    memory_manager::harness_end(mmtk);
}

#[no_mangle]
// We trust the name/value pointer is valid.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn process(
    mmtk: &'static mut MMTK<V8>,
    name: *const c_char,
    value: *const c_char,
) -> bool {
    let name_str: &CStr = unsafe { CStr::from_ptr(name) };
    let value_str: &CStr = unsafe { CStr::from_ptr(value) };
    let res = memory_manager::process(
        mmtk,
        name_str.to_str().unwrap(),
        value_str.to_str().unwrap(),
    );

    res
}

#[no_mangle]
pub extern "C" fn starting_heap_address() -> Address {
    memory_manager::starting_heap_address()
}

#[no_mangle]
pub extern "C" fn last_heap_address() -> Address {
    memory_manager::last_heap_address()
}
