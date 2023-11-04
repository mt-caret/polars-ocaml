use ocaml_interop::{BoxRoot, DynBox, FromOCaml, OCaml, OCamlInt, OCamlRef, OCamlRuntime, ToOCaml};
use std::any::type_name;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::marker::PhantomData;

pub fn dyn_box<'a, T, F, R>(
    cr: &'a mut OCamlRuntime,
    var: OCamlRef<DynBox<T>>,
    body: F,
) -> OCaml<'a, DynBox<R>>
where
    F: FnOnce(T) -> R,
    T: Clone + 'static,
    R: 'static,
{
    let Abstract(rust) = var.to_rust(cr);
    OCaml::box_value(cr, body(rust))
}

pub fn dyn_box2<'a, T1, T2, F, R>(
    cr: &'a mut OCamlRuntime,
    var1: OCamlRef<DynBox<T1>>,
    var2: OCamlRef<DynBox<T2>>,
    body: F,
) -> OCaml<'a, DynBox<R>>
where
    F: FnOnce(T1, T2) -> R,
    T1: Clone + 'static,
    T2: Clone + 'static,
    R: 'static,
{
    let Abstract(t1) = var1.to_rust(cr);
    let Abstract(t2) = var2.to_rust(cr);
    OCaml::box_value(cr, body(t1, t2))
}

pub fn dyn_box_result<'a, T, F, R, E>(
    cr: &'a mut OCamlRuntime,
    var: OCamlRef<DynBox<T>>,
    body: F,
) -> OCaml<'a, Result<DynBox<R>, String>>
where
    F: FnOnce(T) -> Result<R, E>,
    T: Clone + 'static,
    R: 'static,
    E: std::string::ToString,
    Result<Abstract<R>, String>: ToOCaml<Result<DynBox<R>, String>>,
{
    let Abstract(rust) = var.to_rust(cr);
    body(rust)
        .map(Abstract)
        .map_err(|err| err.to_string())
        .to_ocaml(cr)
}

pub fn dyn_box_result_with_cr<'a, T, F, R, E>(
    cr: &'a mut OCamlRuntime,
    var: OCamlRef<DynBox<T>>,
    body: F,
) -> OCaml<'a, Result<DynBox<R>, String>>
where
    F: FnOnce(&mut OCamlRuntime, T) -> Result<R, E>,
    T: Clone + 'static,
    R: 'static,
    E: std::string::ToString,
    Result<Abstract<R>, String>: ToOCaml<Result<DynBox<R>, String>>,
{
    let Abstract(rust) = var.to_rust(cr);
    body(cr, rust)
        .map(Abstract)
        .map_err(|err| err.to_string())
        .to_ocaml(cr)
}

pub fn dyn_box_result2<'a, T1, T2, F, R, E>(
    cr: &'a mut OCamlRuntime,
    v1: OCamlRef<DynBox<T1>>,
    v2: OCamlRef<DynBox<T2>>,
    body: F,
) -> OCaml<'a, Result<DynBox<R>, String>>
where
    F: FnOnce(T1, T2) -> Result<R, E>,
    T1: Clone + 'static,
    T2: Clone + 'static,
    R: 'static,
    E: std::string::ToString,
    Result<Abstract<R>, String>: ToOCaml<Result<DynBox<R>, String>>,
{
    let Abstract(rust1) = v1.to_rust(cr);
    let Abstract(rust2) = v2.to_rust(cr);
    body(rust1, rust2)
        .map(Abstract)
        .map_err(|err| err.to_string())
        .to_ocaml(cr)
}

macro_rules! dyn_box_op {
    ($name:ident, $type:ty, |$($var:ident),+| $body:expr) => {
        #[ocaml_interop_export]
        fn $name(
            cr: &mut &mut OCamlRuntime,
            $(
                $var: OCamlRef<DynBox<$type>>,
            )+
        ) -> OCaml<DynBox<$type>> {
        {
            $(
                let Abstract($var) = $var.to_rust(cr);
            )+

            OCaml::box_value(cr, $body)
        }
        }
    }
}
macro_rules! dyn_box_op_result {
    ($name:ident, $type:ty, |$($var:ident),+| $body:expr) => {
        #[ocaml_interop_export]
        fn $name(
            cr: &mut &mut OCamlRuntime,
            $(
                $var: OCamlRef<DynBox<$type>>,
            )+
        ) -> OCaml<Result<DynBox<$type>, String>> {
        {
            $(
                let Abstract($var) = $var.to_rust(cr);
            )+

            $body.map(Abstract).map_err(|err| err.to_string()).to_ocaml(cr)
        }
        }
    }
}

macro_rules! dyn_box_to_string {
    ($name:ident, $type:ty) => {
        #[ocaml_interop_export]
        fn $name(cr: &mut &mut OCamlRuntime, value: OCamlRef<DynBox<$type>>) -> OCaml<String> {
            let Abstract(value) = value.to_rust(cr);

            value.to_string().to_ocaml(cr)
        }
    };
}

pub(crate) use dyn_box_op;
pub(crate) use dyn_box_op_result;
pub(crate) use dyn_box_to_string;

// This function is actually quite unsafe; as a general rule, additional use of
// this is strongly discouraged. See comment for `raise_ocaml_exception` in the
// implementation of `ocaml_interop_backtrace_support` for more details.
//
// TODO: we unfortunately can't use `ocaml_sys::caml_failwith_value` which would
// prevent us from leaking memory since `cr` isn't accessible in `from_ocaml`
// calls which are where this function is being used (and I'm not sure
// recovering the runtime in these place is safe).
pub unsafe fn ocaml_failwith(error_message: &str) -> ! {
    let error_message = std::ffi::CString::new(error_message).expect("CString::new failed");
    unsafe {
        ocaml_sys::caml_failwith(error_message.as_ptr());
    }
    unreachable!("caml_failwith should never return")
}

polars_ocaml_macros::ocaml_interop_backtrace_support!();

// TODO: add this to ocaml-interop?
pub struct OCamlUniformArray<A> {
    _marker: PhantomData<A>,
}

unsafe impl<A, OCamlA> FromOCaml<OCamlUniformArray<OCamlA>> for Vec<A>
where
    A: FromOCaml<OCamlA>,
{
    fn from_ocaml(v: OCaml<OCamlUniformArray<OCamlA>>) -> Self {
        let size = unsafe { ocaml_sys::wosize_val(v.raw()) };

        // tuple/record/array tag, note that we do not expect a double array
        // tag, since uniform array guarantee boxing.
        assert_eq!(v.tag_value(), 0);

        let mut vec = Vec::with_capacity(size);
        for i in 0..size {
            vec.push(OCaml::<_>::to_rust(&unsafe { v.field(i) }));
        }
        vec
    }
}

// TODO: add this to ocaml-interop?
pub struct OCamlFloatArray {}

unsafe impl FromOCaml<OCamlFloatArray> for Vec<f64> {
    fn from_ocaml(v: OCaml<OCamlFloatArray>) -> Self {
        let size = unsafe { ocaml_sys::wosize_val(v.raw()) };

        // an empty floatarray doesn't have the double array tag, but otherwise
        // we always expect an unboxed float array.
        if size > 0 {
            assert_eq!(v.tag_value(), ocaml_sys::DOUBLE_ARRAY)
        };

        let mut vec = Vec::with_capacity(size);
        for i in 0..size {
            vec.push(unsafe { ocaml_sys::caml_sys_double_field(v.raw(), i) });
        }
        vec
    }
}

pub struct OCamlInt63(pub i64);

unsafe impl FromOCaml<OCamlInt63> for OCamlInt63 {
    fn from_ocaml(v: OCaml<OCamlInt63>) -> Self {
        if v.is_block() {
            let int64 = {
                let val = unsafe { ocaml_sys::field(v.raw(), 1) };
                unsafe { *(val as *const i64) }
            };

            // Base's implementation of `Int63.t` on 32bit platforms is `Int64.t`
            // (a block holding an i64) shifted left with lower bit 0 to match
            // the semantics of `int` on 64bit platforms.
            OCamlInt63(int64 >> 1)
        } else {
            // On 64bit platforms, `Int63.t` is just a regular old OCaml integer.
            OCamlInt63(unsafe { ocaml_sys::int_val(v.raw()) as i64 })
        }
    }
}

#[derive(Debug, Clone)]
pub struct Abstract<T>(pub T);

impl<T> Abstract<T> {
    pub fn get(self) -> T {
        self.0
    }
}

unsafe impl<T: 'static + Clone> FromOCaml<DynBox<T>> for Abstract<T> {
    fn from_ocaml(v: OCaml<DynBox<T>>) -> Self {
        Abstract(Borrow::<T>::borrow(&v).clone())
    }
}

unsafe impl<T: 'static + Clone> ToOCaml<DynBox<T>> for Abstract<T> {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, DynBox<T>> {
        // TODO: I don't fully understand why ToOCaml takes a &self, since that
        // prevents us from using box_value without a clone() call.
        OCaml::box_value(cr, self.0.clone())
    }
}

// TODO: perhaps ocaml_interop can expose the underlying boxroot value
// (along with BoxRoot::new()), so that we don't need to lie and can just use
// that?

// DummyBoxRoot represents a value BoxRoot<T> which has been coerced into a
// BoxRoot<DummyBoxRoot>. This explicitly circumvents the type safety provided
// by ocaml_interop's types, but is necessary if we want to take or return
// values with types which are dependent on GADT arguments.
pub struct DummyBoxRoot(BoxRoot<DummyBoxRoot>);

unsafe impl FromOCaml<DummyBoxRoot> for DummyBoxRoot {
    fn from_ocaml(v: OCaml<DummyBoxRoot>) -> Self {
        DummyBoxRoot(v.root())
    }
}

unsafe impl ToOCaml<DummyBoxRoot> for DummyBoxRoot {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, DummyBoxRoot> {
        self.0.get(cr)
    }
}

impl DummyBoxRoot {
    pub unsafe fn new<T>(boxroot: BoxRoot<T>) -> Self {
        // It's quite unfortunate that we have to transmute here. Ideally we
        // would coerce the type like we do in `interpret` below, but there is
        // no such interface for BoxRoots so we can't do that.
        //
        // The type here is a phantom type so transmute (hopefully) should be safe.
        let boxroot: BoxRoot<DummyBoxRoot> = std::mem::transmute(boxroot);

        DummyBoxRoot(boxroot)
    }

    pub fn interpret<'a, T>(&self, cr: &'a OCamlRuntime) -> OCaml<'a, T> {
        let ocaml_value: OCaml<DummyBoxRoot> = self.0.get(cr);

        unsafe { OCaml::new(cr, ocaml_value.raw()) }
    }
}

pub struct OCamlIntable<T>(pub T);

unsafe impl<T> ToOCaml<OCamlInt> for OCamlIntable<T>
where
    T: TryInto<i64> + Copy,
    <T as TryInto<i64>>::Error: Debug,
{
    fn to_ocaml<'a>(&self, _cr: &'a mut OCamlRuntime) -> OCaml<'a, OCamlInt> {
        OCaml::of_i64(self.0.try_into().expect("Couldn't convert to i64"))
            .expect("Number couldn't fit in OCaml integer")
    }
}

// Coerce<OCamlType, Via, T>, given OCamlType which can be converted into a Rust
// type Via, will try_into() T and will raise an OCaml exception if the
// conversion fails. For example, Coerce<OCamlInt, i64, u32> will convert an
// OCamlInt into an i64 and then try to convert that i64 into a u32.
pub struct Coerce<OCamlType, Via, T>(
    pub Result<T, String>,
    pub PhantomData<Via>,
    pub PhantomData<OCamlType>,
);
impl<OCamlType, Via, T> Coerce<OCamlType, Via, T> {
    pub fn get(self) -> Result<T, String> {
        self.0
    }
}
unsafe impl<OCamlType, Via, T> FromOCaml<OCamlType> for Coerce<OCamlType, Via, T>
where
    Via: FromOCaml<OCamlType>,
    T: TryFrom<Via>,
    <T as TryFrom<Via>>::Error: std::fmt::Debug,
{
    fn from_ocaml(v: OCaml<OCamlType>) -> Self {
        let try_into_result = T::try_from(v.to_rust::<Via>()).map_err(|e| {
            format!(
                "Failed to convert OCaml<{}> (from {}) to Rust<{}>: {:?}",
                type_name::<Via>(),
                type_name::<OCamlType>(),
                type_name::<T>(),
                e
            )
        });

        Coerce(try_into_result, PhantomData, PhantomData)
    }
}

unsafe impl<OCamlType, Via, T> FromOCaml<Option<OCamlType>>
    for Coerce<OCamlType, Option<Via>, Option<T>>
where
    Via: FromOCaml<OCamlType>,
    T: TryFrom<Via>,
    <T as TryFrom<Via>>::Error: std::fmt::Debug,
{
    fn from_ocaml(v: OCaml<Option<OCamlType>>) -> Self {
        let try_into_result =
        match v.to_rust::<Option<Via>>() {
            None => Ok(None),
            Some(v) => match T::try_from(v) {
                Ok(v) => Ok(Some(v)),
                Err(e) => Err(format!(
                        "Failed to convert OCaml<Option<{}>> (from Option<{}>) to Rust<Option<{}>>: {:?}",
                        type_name::<Via>(),
                        type_name::<OCamlType>(),
                        type_name::<T>(),
                        e
                    )),
            },
        };

        Coerce(try_into_result, PhantomData, PhantomData)
    }
}

pub fn unwrap_abstract_vec<T>(v: Vec<Abstract<T>>) -> Vec<T> {
    v.into_iter().map(|Abstract(v)| v).collect()
}
