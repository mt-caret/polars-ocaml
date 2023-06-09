use ocaml_interop::{
    ocaml_alloc_polymorphic_variant, ocaml_alloc_tagged_block, ocaml_alloc_variant,
    ocaml_unpack_polymorphic_variant, ocaml_unpack_variant, polymorphic_variant_tag_hash, DynBox,
    FromOCaml, OCaml, OCamlInt, OCamlList, OCamlRuntime, ToOCaml,
};
use polars::{lazy::dsl::WindowMapping, prelude::*};
use smartstring::{LazyCompact, SmartString};
use std::borrow::Borrow;

pub unsafe fn ocaml_failwith(error_message: &str) -> ! {
    let error_message = std::ffi::CString::new(error_message).expect("CString::new failed");
    unsafe {
        ocaml_sys::caml_failwith(error_message.as_ptr());
    }
    unreachable!("caml_failwith should never return")
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
                DataType::Unknown,
            }
        };
        PolarsDataType(result.expect("Failure when unpacking an OCaml<DataType> variant into PolarsDataType (unexpected tag value"))
    }
}

unsafe fn ocaml_value<'a, T>(cr: &'a mut OCamlRuntime, n: i32) -> OCaml<'a, T> {
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
                    ocaml_alloc_tagged_block!(cr, 0, timeunit : TimeUnit, timezone: Option<String>)
                }
                DataType::Duration(timeunit) => {
                    let timeunit = PolarsTimeUnit(*timeunit);
                    ocaml_alloc_tagged_block!(cr, 1,  timeunit: TimeUnit)
                }
                DataType::Time => ocaml_value(cr, 14),
                DataType::List(datatype) => {
                    let datatype = PolarsDataType(*datatype.clone());
                    ocaml_alloc_tagged_block!(cr, 2,  datatype: DataType)
                }
                DataType::Null => ocaml_value(cr, 15),
                DataType::Unknown => ocaml_value(cr, 16),
            }
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
                    ocaml_alloc_tagged_block!(cr, 0, upto : Option<OCamlInt>)
                }
                FillNullStrategy::Forward(upto) => {
                    let upto = upto.map(|upto| upto as i64);
                    ocaml_alloc_tagged_block!(cr, 1, upto : Option<OCamlInt>)
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

pub fn unwrap_abstract_vec<T>(v: Vec<Abstract<T>>) -> Vec<T> {
    v.into_iter().map(|Abstract(v)| v).collect()
}
