use libc::c_void;
use mmtk::vm::ReferenceGlue;
use mmtk::util::ObjectReference;
use mmtk::TraceLocal;
use mmtk::util::OpaquePointer;
use V8;

pub struct VMReferenceGlue {}

impl ReferenceGlue<V8> for VMReferenceGlue {
    fn set_referent(reff: ObjectReference, referent: ObjectReference) {
        unimplemented!()
    }
    fn get_referent(object: ObjectReference) -> ObjectReference {
        unimplemented!()
    }
    fn process_reference<T: TraceLocal>(trace: &mut T, reference: ObjectReference, tls: OpaquePointer) -> ObjectReference {
        unimplemented!()
    }
}