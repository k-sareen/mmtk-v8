#![feature(vec_into_raw_parts)]

extern crate libc;
extern crate mmtk;
#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

use std::ptr::null_mut;

use libc::c_void;
use mmtk::scheduler::GCWorker;
use mmtk::util::ObjectReference;
use mmtk::util::OpaquePointer;
use mmtk::vm::VMBinding;
use mmtk::Mutator;
use mmtk::MMTK;
pub mod active_plan;
pub mod api;
pub mod collection;
mod object_archive;
pub mod object_model;
pub mod reference_glue;
pub mod scanning;
use mmtk::util::{Address};

#[repr(C)]
pub struct NewBuffer {
    pub ptr: *mut Address,
    pub capacity: usize,
}

type ProcessEdgesFn = *const extern "C" fn(buf: *mut Address, size: usize, cap: usize) -> NewBuffer;

#[repr(C)]
pub struct V8_Upcalls {
    pub stop_all_mutators: extern "C" fn(tls: OpaquePointer),
    pub resume_mutators: extern "C" fn(tls: OpaquePointer),
    pub spawn_worker_thread: extern "C" fn(tls: OpaquePointer, ctx: *mut GCWorker<V8>),
    pub block_for_gc: extern "C" fn(),
    pub active_collector: extern "C" fn(tls: OpaquePointer) -> *mut GCWorker<V8>,
    pub get_next_mutator: extern "C" fn() -> *mut Mutator<V8>,
    pub reset_mutator_iterator: extern "C" fn(),
    pub compute_static_roots: extern "C" fn(trace: *mut c_void, tls: OpaquePointer),
    pub compute_global_roots: extern "C" fn(trace: *mut c_void, tls: OpaquePointer),
    pub compute_thread_roots: extern "C" fn(trace: *mut c_void, tls: OpaquePointer),
    pub scan_object: extern "C" fn(trace: *mut c_void, object: ObjectReference, tls: OpaquePointer),
    pub dump_object: extern "C" fn(object: ObjectReference),
    pub get_object_size: extern "C" fn(object: ObjectReference) -> usize,
    pub get_mmtk_mutator: extern "C" fn(tls: OpaquePointer) -> *mut Mutator<V8>,
    pub is_mutator: extern "C" fn(tls: OpaquePointer) -> bool,
    pub scan_roots: extern "C" fn(process_edges: ProcessEdgesFn),
    pub scan_objects: extern "C" fn(objects: *const ObjectReference, count: usize, process_edges: ProcessEdgesFn),
}

pub static mut UPCALLS: *const V8_Upcalls = null_mut();

#[derive(Default)]
pub struct V8;

impl VMBinding for V8 {
    type VMObjectModel = object_model::VMObjectModel;
    type VMScanning = scanning::VMScanning;
    type VMCollection = collection::VMCollection;
    type VMActivePlan = active_plan::VMActivePlan;
    type VMReferenceGlue = reference_glue::VMReferenceGlue;

    const MAX_ALIGNMENT: usize = 32;
}

lazy_static! {
    pub static ref SINGLETON: MMTK<V8> = {
        MMTK::new()
    };
}
