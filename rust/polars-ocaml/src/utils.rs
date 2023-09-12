use ocaml_interop::{
    impl_from_ocaml_variant, ocaml_alloc_polymorphic_variant, ocaml_alloc_tagged_block,
    ocaml_alloc_variant, ocaml_unpack_polymorphic_variant, ocaml_unpack_variant,
    polymorphic_variant_tag_hash, BoxRoot, DynBox, FromOCaml, OCaml, OCamlInt, OCamlList,
    OCamlRuntime, ToOCaml,
};
use polars::series::IsSorted;
use polars::{lazy::dsl::WindowMapping, prelude::*};
use smartstring::{LazyCompact, SmartString};
use std::any::type_name;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::marker::PhantomData;

macro_rules! dyn_box {
    ($cr:ident, |$($var:ident),+| $body:expr) => {
        {
            $(
                let Abstract($var) = $var.to_rust($cr);
            )+

            OCaml::box_value($cr, $body)
        }
    };
}

macro_rules! dyn_box_result {
    ($cr:ident, |$($var:ident),+| $body:expr) => {
        {
            $(
                let Abstract($var) = $var.to_rust($cr);
            )+

            $body.map(Abstract).map_err(|err| err.to_string()).to_ocaml($cr)
        }
    };
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
            dyn_box!(cr, |$($var),+| $body)
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
            dyn_box_result!(cr, |$($var),+| $body)
        }
    }
}

pub(crate) use dyn_box;
pub(crate) use dyn_box_op;
pub(crate) use dyn_box_op_result;
pub(crate) use dyn_box_result;

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

pub struct PolarsTimeUnit(pub TimeUnit);

unsafe impl FromOCaml<TimeUnit> for PolarsTimeUnit {
    fn from_ocaml(v: OCaml<TimeUnit>) -> Self {
        let result = ocaml_unpack_variant! {
            v => {
                TimeUnit::Nanoseconds,
                TimeUnit::Microseconds,
                TimeUnit::Milliseconds,
            }
        };
        PolarsTimeUnit(result.expect("Failure when unpacking an OCaml<TimeUnit> variant into PolarsTimeUnit (unexpected tag value"))
    }
}

unsafe impl ToOCaml<TimeUnit> for PolarsTimeUnit {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, TimeUnit> {
        let PolarsTimeUnit(timeunit) = self;
        ocaml_alloc_variant! {
            cr, timeunit => {
                TimeUnit::Nanoseconds,
                TimeUnit::Microseconds,
                TimeUnit::Milliseconds,
            }
        }
    }
}

pub struct PolarsDataType(pub DataType);

unsafe impl FromOCaml<DataType> for PolarsDataType {
    fn from_ocaml(v: OCaml<DataType>) -> Self {
        let result = ocaml_unpack_variant! {
            v => {
                DataType::Boolean,
                DataType::UInt8,
                DataType::UInt16,
                DataType::UInt32,
                DataType::UInt64,
                DataType::Int8,
                DataType::Int16,
                DataType::Int32,
                DataType::Int64,
                DataType::Float32,
                DataType::Float64,
                DataType::Utf8,
                DataType::Binary,
                DataType::Date,
                DataType::Datetime(timeunit: TimeUnit, timezone: Option<String>) => {
                    let PolarsTimeUnit(timeunit) = timeunit;
                    DataType::Datetime(timeunit, timezone)},
                DataType::Duration(timeunit: TimeUnit) => {
                    let PolarsTimeUnit(timeunit) = timeunit;
                    DataType::Duration(timeunit)},
                DataType::Time,
                DataType::List(datatype: DataType) => {
                    let PolarsDataType(datatype) = datatype;
                    DataType::List(Box::new(datatype))
                },
                DataType::Null,
                DataType::Struct(fields: OCamlList<(String, DataType)>) => {
                    let fields_: Vec<(String, PolarsDataType)> = fields;
                    let fields: Vec<Field> =
                        fields_
                        .into_iter()
                        .map(|(name, PolarsDataType(datatype))| Field { name: SmartString::from(name), dtype: datatype })
                        .collect();
                    DataType::Struct(fields)
                },
                DataType::Unknown,
            }
        };
        PolarsDataType(result.expect("Failure when unpacking an OCaml<DataType> variant into PolarsDataType (unexpected tag value"))
    }
}

unsafe fn ocaml_value<T>(cr: &mut OCamlRuntime, n: i32) -> OCaml<T> {
    unsafe { OCaml::new(cr, OCaml::of_i32(n).raw()) }
}

unsafe impl ToOCaml<DataType> for PolarsDataType {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, DataType> {
        let PolarsDataType(datatype) = self;
        // We expand out the macro here since we need to do some massaging of the
        // values to get things into the right shape to convert to OCaml values
        unsafe {
            match datatype {
                DataType::Boolean => ocaml_value(cr, 0),
                DataType::UInt8 => ocaml_value(cr, 1),
                DataType::UInt16 => ocaml_value(cr, 2),
                DataType::UInt32 => ocaml_value(cr, 3),
                DataType::UInt64 => ocaml_value(cr, 4),
                DataType::Int8 => ocaml_value(cr, 5),
                DataType::Int16 => ocaml_value(cr, 6),
                DataType::Int32 => ocaml_value(cr, 7),
                DataType::Int64 => ocaml_value(cr, 8),
                DataType::Float32 => ocaml_value(cr, 9),
                DataType::Float64 => ocaml_value(cr, 10),
                DataType::Utf8 => ocaml_value(cr, 11),
                DataType::Binary => ocaml_value(cr, 12),
                DataType::Date => ocaml_value(cr, 13),
                DataType::Datetime(timeunit, timezone) => {
                    let timeunit = PolarsTimeUnit(*timeunit);
                    let timezone = timezone.clone();
                    ocaml_alloc_tagged_block!(cr, 0, timeunit: TimeUnit, timezone: Option<String>)
                }
                DataType::Duration(timeunit) => {
                    let timeunit = PolarsTimeUnit(*timeunit);
                    ocaml_alloc_tagged_block!(cr, 1, timeunit: TimeUnit)
                }
                DataType::Time => ocaml_value(cr, 14),
                DataType::List(datatype) => {
                    let datatype = PolarsDataType(*datatype.clone());
                    ocaml_alloc_tagged_block!(cr, 2, datatype: DataType)
                }
                DataType::Null => ocaml_value(cr, 15),
                DataType::Struct(fields) => {
                    let fields: Vec<(String, PolarsDataType)> = fields
                        .iter()
                        .map(|field| (field.name.to_string(), PolarsDataType(field.dtype.clone())))
                        .collect();
                    ocaml_alloc_tagged_block!(cr, 3, fields: OCamlList<(String, DataType)>)
                }
                DataType::Unknown => ocaml_value(cr, 16),
            }
        }
    }
}

#[derive(Debug)]
pub enum GADTDataType {
    Boolean,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Int8,
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,
    Utf8,
    Binary,
    List(Box<GADTDataType>),
}

impl_from_ocaml_variant! {
    GADTDataType {
        GADTDataType::Boolean,
        GADTDataType::UInt8,
        GADTDataType::UInt16,
        GADTDataType::UInt32,
        GADTDataType::UInt64,
        GADTDataType::Int8,
        GADTDataType::Int16,
        GADTDataType::Int32,
        GADTDataType::Int64,
        GADTDataType::Float32,
        GADTDataType::Float64,
        GADTDataType::Utf8,
        GADTDataType::Binary,
        GADTDataType::List(data_type: GADTDataType),
    }
}

impl GADTDataType {
    pub fn to_data_type(&self) -> DataType {
        match self {
            GADTDataType::Boolean => DataType::Boolean,
            GADTDataType::UInt8 => DataType::UInt8,
            GADTDataType::UInt16 => DataType::UInt16,
            GADTDataType::UInt32 => DataType::UInt32,
            GADTDataType::UInt64 => DataType::UInt64,
            GADTDataType::Int8 => DataType::Int8,
            GADTDataType::Int16 => DataType::Int16,
            GADTDataType::Int32 => DataType::Int32,
            GADTDataType::Int64 => DataType::Int64,
            GADTDataType::Float32 => DataType::Float32,
            GADTDataType::Float64 => DataType::Float64,
            GADTDataType::Utf8 => DataType::Utf8,
            GADTDataType::Binary => DataType::Binary,
            GADTDataType::List(data_type) => DataType::List(Box::new(data_type.to_data_type())),
        }
    }
}

pub struct PolarsFillNullStrategy(pub FillNullStrategy);

unsafe impl FromOCaml<FillNullStrategy> for PolarsFillNullStrategy {
    fn from_ocaml(v: OCaml<FillNullStrategy>) -> Self {
        let result = ocaml_unpack_variant! {
            v => {
                FillNullStrategy::Backward(upto: Option<OCamlInt>) => {
                    let upto_: Option<i64> = upto;
                    let upto: Option<Option<u32>> = upto_.map(|upto| TryInto::<u32>::try_into(upto).ok());
                    match upto {
                        None => FillNullStrategy::Backward(None),
                        Some(None) => unsafe { ocaml_failwith(&format!("Failed conversion to u32 {:?}", upto_)) },
                        Some(upto) => FillNullStrategy::Backward(upto),
                    }
                },
                FillNullStrategy::Forward(upto: Option<OCamlInt>) => {
                    let upto_: Option<i64> = upto;
                    let upto: Option<Option<u32>> = upto_.map(|upto| TryInto::<u32>::try_into(upto).ok());
                    match upto {
                        None => FillNullStrategy::Forward(None),
                        Some(None) => unsafe { ocaml_failwith(&format!("Failed conversion to u32 {:?}", upto_)) },
                        Some(upto) => FillNullStrategy::Forward(upto),
                    }
                },
                FillNullStrategy::Mean,
                FillNullStrategy::Min,
                FillNullStrategy::Max,
                FillNullStrategy::Zero,
                FillNullStrategy::One,
                FillNullStrategy::MaxBound,
                FillNullStrategy::MinBound,
            }
        };
        PolarsFillNullStrategy(result.expect("Failure when unpacking an OCaml<FillNullStrategy> variant into PolarsFillNullStrategy (unexpected tag value"))
    }
}

unsafe impl ToOCaml<FillNullStrategy> for PolarsFillNullStrategy {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, FillNullStrategy> {
        let PolarsFillNullStrategy(fill_null_strategy) = self;

        // We expand out the macro here since we need to do some massaging of the
        // values to get things into the right shape to convert to OCaml values
        unsafe {
            match fill_null_strategy {
                FillNullStrategy::Backward(upto) => {
                    let upto = upto.map(|upto| upto as i64);
                    ocaml_alloc_tagged_block!(cr, 0, upto: Option<OCamlInt>)
                }
                FillNullStrategy::Forward(upto) => {
                    let upto = upto.map(|upto| upto as i64);
                    ocaml_alloc_tagged_block!(cr, 1, upto: Option<OCamlInt>)
                }
                FillNullStrategy::Mean => ocaml_value(cr, 0),
                FillNullStrategy::Min => ocaml_value(cr, 1),
                FillNullStrategy::Max => ocaml_value(cr, 2),
                FillNullStrategy::Zero => ocaml_value(cr, 3),
                FillNullStrategy::One => ocaml_value(cr, 4),
                FillNullStrategy::MaxBound => ocaml_value(cr, 5),
                FillNullStrategy::MinBound => ocaml_value(cr, 6),
            }
        }
    }
}

pub struct PolarsInterpolationMethod(pub InterpolationMethod);

unsafe impl FromOCaml<InterpolationMethod> for PolarsInterpolationMethod {
    fn from_ocaml(v: OCaml<InterpolationMethod>) -> Self {
        let result = ocaml_unpack_polymorphic_variant! {
            v => {
                Linear => InterpolationMethod::Linear,
                Nearest => InterpolationMethod::Nearest,
            }
        };
        PolarsInterpolationMethod(result.expect("Failure when unpacking an OCaml<InterpolationMethod> variant into PolarsInterpolationMethod (unexpected tag value"))
    }
}

unsafe impl ToOCaml<InterpolationMethod> for PolarsInterpolationMethod {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, InterpolationMethod> {
        let PolarsInterpolationMethod(interpolation_method) = self;

        ocaml_alloc_polymorphic_variant! {
            cr, interpolation_method => {
                InterpolationMethod::Linear,
                InterpolationMethod::Nearest,
            }
        }
    }
}

pub struct PolarsWindowMapping(pub WindowMapping);

unsafe impl FromOCaml<WindowMapping> for PolarsWindowMapping {
    fn from_ocaml(v: OCaml<WindowMapping>) -> Self {
        let result = ocaml_unpack_polymorphic_variant! {
            v => {
                Groups_to_rows => WindowMapping::GroupsToRows,
                Explode => WindowMapping::Explode,
                Join => WindowMapping::Join,
            }
        };
        PolarsWindowMapping(result.expect("Failure when unpacking an OCaml<WindowMapping> variant into PolarsWindowMapping (unexpected tag value"))
    }
}

unsafe impl ToOCaml<WindowMapping> for PolarsWindowMapping {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, WindowMapping> {
        let PolarsWindowMapping(window_mapping) = self;

        unsafe {
            match window_mapping {
                WindowMapping::GroupsToRows => {
                    OCaml::new(cr, polymorphic_variant_tag_hash!(Groups_to_rows))
                }
                WindowMapping::Explode => OCaml::new(cr, polymorphic_variant_tag_hash!(Explode)),
                WindowMapping::Join => OCaml::new(cr, polymorphic_variant_tag_hash!(Join)),
            }
        }
    }
}

pub struct PolarsRankMethod(pub RankMethod);

unsafe impl FromOCaml<RankMethod> for PolarsRankMethod {
    fn from_ocaml(v: OCaml<RankMethod>) -> Self {
        let result = ocaml_unpack_polymorphic_variant! {
            v => {
                Average => RankMethod::Average,
                Min => RankMethod::Min,
                Max => RankMethod::Max,
                Dense => RankMethod::Dense,
                Ordinal => RankMethod::Ordinal,
                Random => RankMethod::Random,
            }
        };
        PolarsRankMethod(result.expect("Failure when unpacking an OCaml<RankMethod> variant into PolarsRankMethod (unexpected tag value"))
    }
}

unsafe impl ToOCaml<RankMethod> for PolarsRankMethod {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, RankMethod> {
        let PolarsRankMethod(rank_method) = self;
        ocaml_alloc_polymorphic_variant! {
            cr, rank_method => {
                RankMethod::Average,
                RankMethod::Min,
                RankMethod::Max,
                RankMethod::Dense,
                RankMethod::Ordinal,
                RankMethod::Random,
            }
        }
    }
}

pub struct PolarsJoinType(pub JoinType);
unsafe impl FromOCaml<JoinType> for PolarsJoinType {
    fn from_ocaml(v: OCaml<JoinType>) -> Self {
        let result = ocaml_unpack_variant! {
            v => {
                JoinType::Left,
                JoinType::Inner,
                JoinType::Outer,
                JoinType::AsOf(dummy: ()) => {
                    // We don't actually care about the value of dummy, we just
                    // need to make sure that the variant is treated as a
                    // block, not a value.
                    let () = dummy;

                    unsafe {
                        let strategy = v.field::<AsofStrategy>(0);
                        let strategy = ocaml_unpack_polymorphic_variant! {
                            strategy => {
                                Backward => AsofStrategy::Backward,
                                Forward => AsofStrategy::Forward,
                                Nearest => AsofStrategy::Nearest,
                            }
                        };
                        let strategy = strategy.expect(
                            "Failure when unpacking an OCaml<AsofStrategy> variant (unexpected tag value",
                        );

                        let tolerance: Option<String> = v.field::<Option<String>>(1).to_rust();
                        let tolerance = tolerance.map(SmartString::from);

                        let left_by: Option<Vec<String>> = v.field::<Option<OCamlList<String>>>(2).to_rust();
                        let left_by: Option<Vec<SmartString<LazyCompact>>> =
                            left_by.map(|left_by| left_by.into_iter().map(SmartString::from).collect());

                        let right_by: Option<Vec<String>> = v.field::<Option<OCamlList<String>>>(3).to_rust();
                        let right_by: Option<Vec<SmartString<LazyCompact>>> =
                            right_by.map(|right_by| right_by.into_iter().map(SmartString::from).collect());

                        JoinType::AsOf(AsOfOptions {
                            strategy,
                            tolerance: None,
                            tolerance_str: tolerance,
                            left_by,
                            right_by,
                        })
                    }
                },
                JoinType::Cross,
                JoinType::Semi,
                JoinType::Anti,
            }
        };

        PolarsJoinType(result.expect("Failure when unpacking an OCaml<JoinType> variant into PolarsJoinType (unexpected tag value"))
    }
}

pub struct PolarsClosedWindow(pub ClosedWindow);

unsafe impl FromOCaml<ClosedWindow> for PolarsClosedWindow {
    fn from_ocaml(v: OCaml<ClosedWindow>) -> Self {
        let result = ocaml_unpack_polymorphic_variant! {
            v => {
                Left => ClosedWindow::Left,
                Right => ClosedWindow::Right,
                Both => ClosedWindow::Both,
                None_ => ClosedWindow::None,
            }
        };
        PolarsClosedWindow(result.expect("Failure when unpacking an OCaml<ClosedWindow> variant into PolarsClosedWindow (unexpected tag value"))
    }
}

pub struct PolarsStartBy(pub StartBy);

unsafe impl FromOCaml<StartBy> for PolarsStartBy {
    fn from_ocaml(v: OCaml<StartBy>) -> Self {
        let result = ocaml_unpack_polymorphic_variant! {
            v => {
                Window_bound => StartBy::WindowBound,
                Data_point => StartBy::DataPoint,
                Monday => StartBy::Monday,
                Tuesday => StartBy::Tuesday,
                Wednesday => StartBy::Wednesday,
                Thursday => StartBy::Thursday,
                Friday => StartBy::Friday,
                Saturday => StartBy::Saturday,
                Sunday => StartBy::Sunday,
            }
        };
        PolarsStartBy(result.expect("Failure when unpacking an OCaml<StartBy> variant into PolarsStartBy (unexpected tag value"))
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

pub struct PolarsIsSorted(pub IsSorted);
unsafe impl FromOCaml<IsSorted> for PolarsIsSorted {
    fn from_ocaml(v: OCaml<IsSorted>) -> Self {
        let result = ocaml_unpack_polymorphic_variant! {
            v => {
                Ascending => IsSorted::Ascending,
                Descending => IsSorted::Descending,
                Not => IsSorted::Not,
            }
        };

        PolarsIsSorted(result.expect("Failure when unpacking an OCaml<IsSorted> variant into PolarsIsSorted (unexpected tag value"))
    }
}

pub struct Abstract<T>(pub T);
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
    pub unsafe fn new<'a, T>(boxroot: BoxRoot<T>) -> Self {
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

pub fn unwrap_abstract_vec<T>(v: Vec<Abstract<T>>) -> Vec<T> {
    v.into_iter().map(|Abstract(v)| v).collect()
}
