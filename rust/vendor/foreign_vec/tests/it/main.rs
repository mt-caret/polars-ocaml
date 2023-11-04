use foreign_vec::ForeignVec;

// say that we have a foreign struct allocated by an external allocator (e.g. C++)
// owning an immutable memory region
#[repr(C)]
struct Foreign {
    ptr: *const i32,
    length: usize,
    // this is usually created at the FFI boundary; `capacity` is usually "hidden" in that
    // it could contain a C++ `shared_ptr` or something else describing the region
    // "on the other side".
    capacity: usize,
}

// whose drop calls an external function that deallocates the region
impl Drop for Foreign {
    fn drop(&mut self) {
        // mocking an external deallocation
        unsafe { Vec::from_raw_parts(self.ptr as *mut i32, self.length, self.capacity) };
    }
}

// The type that we use on the library uses `foreign_vec`
// this could be a generic over `T` when the in-memory format supports multiple types.
type MyForeignVec = ForeignVec<Foreign, i32>;

#[test]
fn test_vec() {
    // we can use it with `Vec`:
    let expected: &[i32] = &[1, 2];

    // when we have a vector, we can use `.into()`
    let vec = expected.to_vec();
    let mut vec: MyForeignVec = vec.into();

    // deref works as expected
    assert_eq!(&*vec, expected);

    // debug works as expected
    assert_eq!(format!("{:?}", vec), "[1, 2]");

    // you can retrieve a mut vec (since it is allocated by Rust)
    assert_eq!(vec.get_vec(), Some(&mut vec![1, 2]));

    // this calls `Vec::drop`, as usual
    drop(vec)
}

// this is just `Vec::into_raw_parts`, which is only available in unstable channels
fn into_raw_parts<T>(vec: Vec<T>) -> (*mut T, usize, usize) {
    let r = (vec.as_ptr() as *mut T, vec.len(), vec.capacity());
    std::mem::forget(vec);
    r
}

#[test]
fn test_foreign() {
    // on an externally allocated pointer (here from Rust, but a foreign call would do the same)
    let expected: &[i32] = &[1, 2];

    let a = expected.to_vec();
    let (ptr, length, capacity) = into_raw_parts(a);
    // this is usually created at the FFI boundary; `capacity` is usually "hidden" in that
    // it could contain a C++ `shared_ptr` instead.
    let a = Foreign {
        ptr,
        length,
        capacity,
    };

    // create a `MyForeignVec` from a foreign that implements `Deref`.
    let mut vec = unsafe { MyForeignVec::from_foreign(a.ptr, a.length, a) };
    assert_eq!(&*vec, expected);
    assert_eq!(vec.get_vec(), None);

    // this calls `Foreign::drop`, which calls the foreign function
    drop(vec);
}
