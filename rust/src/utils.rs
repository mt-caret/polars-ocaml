use ocaml_interop::{
    ocaml_alloc_tagged_block, ocaml_alloc_variant, ocaml_unpack_variant, DynBox, FromOCaml, OCaml,
    OCamlRuntime, ToOCaml,
};
use polars::prelude::*;
use std::borrow::Borrow;

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
