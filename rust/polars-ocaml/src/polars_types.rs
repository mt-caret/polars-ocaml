use crate::interop::*;
use ocaml_interop::{
    impl_from_ocaml_variant, ocaml_alloc_polymorphic_variant, ocaml_alloc_tagged_block,
    ocaml_alloc_variant, ocaml_unpack_polymorphic_variant, ocaml_unpack_variant,
    polymorphic_variant_tag_hash, DynBox, FromOCaml, OCaml, OCamlInt, OCamlList, OCamlRuntime,
    ToOCaml,
};
use polars::prelude::*;
use polars::series::IsSorted;
use smartstring::{LazyCompact, SmartString};

#[derive(Debug, Clone)]
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
                DataType::Datetime(timeunit: TimeUnit, timezone: Option<DynBox<chrono_tz::Tz>>) => {
                    let PolarsTimeUnit(timeunit) = timeunit;
                    let timezone: Option<Abstract<chrono_tz::Tz>> = timezone;
                    DataType::Datetime(timeunit, timezone.map(|tz| tz.get().name().to_string()))},
                DataType::Duration(timeunit: TimeUnit) => {
                    let PolarsTimeUnit(timeunit) = timeunit;
                    DataType::Duration(timeunit)},
                DataType::Time,
                DataType::List(datatype: DataType) => {
                    let PolarsDataType(datatype) = datatype;
                    DataType::List(Box::new(datatype))
                },
                DataType::Null,
                DataType::Categorical(local_rev_mapping_opt: Option<DynBox<Arc<RevMapping>>>) => {
                    let local_rev_mapping_opt: Option<Abstract<Arc<RevMapping>>> = local_rev_mapping_opt;
                    DataType::Categorical(local_rev_mapping_opt.map(Abstract::get))
                },
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

unsafe fn ocaml_value<T>(cr: &OCamlRuntime, n: i32) -> OCaml<T> {
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
                    let timezone = timezone.clone().map(|timezone| {
                        Abstract(
                            timezone
                                .parse::<chrono_tz::Tz>()
                                .expect("unexpected timezone"),
                        )
                    });

                    ocaml_alloc_tagged_block!(cr, 0, timeunit: TimeUnit, timezone: Option<DynBox<chrono_tz::Tz>>)
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
                DataType::Categorical(local_rev_mapping_opt) => {
                    let local_rev_mapping_opt = local_rev_mapping_opt.clone().map(Abstract);
                    ocaml_alloc_tagged_block!(cr, 3, local_rev_mapping_opt: Option<DynBox<Arc<RevMapping>>>)
                }
                DataType::Struct(fields) => {
                    let fields: Vec<(String, PolarsDataType)> = fields
                        .iter()
                        .map(|field| (field.name.to_string(), PolarsDataType(field.dtype.clone())))
                        .collect();
                    ocaml_alloc_tagged_block!(cr, 4, fields: OCamlList<(String, DataType)>)
                }
                DataType::Unknown => ocaml_value(cr, 16),
            }
        }
    }
}

#[derive(Debug, Clone)]
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
    Date,
    Datetime(PolarsTimeUnit, Option<Abstract<chrono_tz::Tz>>),
    Duration(PolarsTimeUnit),
    Time,
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
        GADTDataType::Date,
        GADTDataType::Datetime(time_unit: TimeUnit, time_zone: Option<DynBox<chrono_tz::Tz>>),
        GADTDataType::Duration(time_unit: TimeUnit),
        GADTDataType::Time,
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
            GADTDataType::Date => DataType::Date,
            GADTDataType::Datetime(time_unit, time_zone) => DataType::Datetime(
                time_unit.0,
                time_zone
                    .clone()
                    .map(|time_zone| time_zone.get().name().to_string()),
            ),
            GADTDataType::Duration(time_unit) => DataType::Duration(time_unit.0),
            GADTDataType::Time => DataType::Time,
            GADTDataType::List(data_type) => DataType::List(Box::new(data_type.to_data_type())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PolarsQuantileInterpolOptions(pub QuantileInterpolOptions);

unsafe impl FromOCaml<QuantileInterpolOptions> for PolarsQuantileInterpolOptions {
    fn from_ocaml(v: OCaml<QuantileInterpolOptions>) -> Self {
        let result = ocaml_unpack_variant! {
            v => {
                QuantileInterpolOptions::Nearest,
                QuantileInterpolOptions::Lower,
                QuantileInterpolOptions::Higher,
                QuantileInterpolOptions::Linear,
                QuantileInterpolOptions::Midpoint,
            }
        };
        PolarsQuantileInterpolOptions(result.expect("Failure when unpacking an OCaml<QuantileInterpolOptions> variant into PolarsQuantileInterpolOptions (unexpected tag value"))
    }
}

unsafe impl ToOCaml<QuantileInterpolOptions> for PolarsQuantileInterpolOptions {
    fn to_ocaml<'a>(&self, cr: &'a mut OCamlRuntime) -> OCaml<'a, QuantileInterpolOptions> {
        let PolarsQuantileInterpolOptions(quantile_interpol_options) = self;
        ocaml_alloc_variant! {
            cr, quantile_interpol_options => {
                QuantileInterpolOptions::Nearest,
                QuantileInterpolOptions::Lower,
                QuantileInterpolOptions::Higher,
                QuantileInterpolOptions::Linear,
                QuantileInterpolOptions::Midpoint,
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

#[derive(Debug, Clone)]
pub struct PolarsQuantileInterpolOptions(pub QuantileInterpolOptions);

unsafe impl FromOCaml<QuantileInterpolOptions> for PolarsQuantileInterpolOptions {
    fn from_ocaml(v: OCaml<QuantileInterpolOptions>) -> Self {
        let result = ocaml_unpack_polymorphic_variant! {
            v => {
                Nearest => QuantileInterpolOptions::Nearest,
                Lower => QuantileInterpolOptions::Lower,
                Higher => QuantileInterpolOptions::Higher,
                Linear => QuantileInterpolOptions::Linear,
                Midpoint => QuantileInterpolOptions::Midpoint,
            }
        };
        PolarsQuantileInterpolOptions(result.expect("Failure when unpacking an OCaml<QuantileInterpolOptions> variant into PolarsQuantileInterpolOptions (unexpected tag value"))
    }
}
