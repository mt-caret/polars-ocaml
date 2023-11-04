//! A crate providing generalised value traits for working with
//! `JSONesque` values.
#![warn(unused_extern_crates)]
#![deny(
    clippy::all,
    clippy::unwrap_used,
    clippy::unnecessary_unwrap,
    clippy::pedantic
)]
// We might want to revisit inline_always
#![allow(clippy::module_name_repetitions, clippy::inline_always)]
#![allow(clippy::type_repetition_in_bounds)]
#![deny(missing_docs)]

#[cfg(all(feature = "128bit", feature = "c-abi"))]
compile_error!(
    "Combining the features `128bit` and `c-abi` is impossible because i128's \
    ABI is unstable (see \
    https://github.com/rust-lang/unsafe-code-guidelines/issues/119). Please \
    use only one of them in order to compile this crate. If you don't know \
    where this error is coming from, it's possible that you depend on \
    value-trait twice indirectly, once with the `c-abi` feature, and once with \
    the `128bit` feature, and that they have been merged by Cargo."
);

use std::borrow::{Borrow, Cow};
use std::convert::TryInto;
use std::fmt;
use std::hash::Hash;
use std::io::{self, Write};
use std::ops::{Index, IndexMut};

mod array;
/// Traits for serializing JSON
pub mod generator;
mod node;
mod object;
mod option;
/// Prelude for traits
pub mod prelude;

pub use array::Array;
pub use node::StaticNode;
pub use object::Object;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// An access error for `ValueType`
pub enum AccessError {
    /// An access attempt to a Value was made under the
    /// assumption that it is an Object - the Value however
    /// wasn't.
    NotAnObject,
    /// An access attempt to a Value was made under the
    /// assumption that it is an Array - the Value however
    /// wasn't.
    NotAnArray,
}

#[cfg(not(tarpaulin_include))]
impl fmt::Display for AccessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAnArray => write!(f, "The value is not an array"),
            Self::NotAnObject => write!(f, "The value is not an object"),
        }
    }
}
impl std::error::Error for AccessError {}

/// Extended types that have no native representation in JSON
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedValueType {
    /// A 32 bit signed integer value
    I32,
    /// A 16 bit signed integer value
    I16,
    /// A 8 bit signed integer value
    I8,
    /// A 32 bit unsigned integer value
    U32,
    /// A 16 bit unsigned integer value
    U16,
    /// A 8 bit unsigned integer value
    U8,
    /// A useize value
    Usize,
    /// A 32 bit floating point value
    F32,
    /// A single utf-8 character
    Char,
    /// Not a value at all
    None,
}

impl Default for ExtendedValueType {
    fn default() -> Self {
        Self::None
    }
}

impl fmt::Display for ExtendedValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::I32 => write!(f, "i32"),
            Self::I16 => write!(f, "i16"),
            Self::I8 => write!(f, "i8"),
            Self::U32 => write!(f, "u32"),
            Self::U16 => write!(f, "u16"),
            Self::U8 => write!(f, "u8"),
            Self::Usize => write!(f, "usize"),
            Self::F32 => write!(f, "f32"),
            Self::Char => write!(f, "char"),
            Self::None => write!(f, "none"),
        }
    }
}

/// Types of JSON values
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ValueType {
    /// null
    Null,
    /// a boolean
    Bool,
    /// a signed integer type
    I64,
    /// a 128 bit signed integer
    I128,
    /// a unsigned integer type
    U64,
    /// a 128 bit unsiged integer
    U128,
    /// a float type
    F64,
    /// a string type
    String,
    /// an array
    Array,
    /// an object
    Object,
    /// Extended types that do not have a real representation in JSON
    Extended(ExtendedValueType),
    #[cfg(feature = "custom-types")]
    /// a custom type
    Custom(&'static str),
}
impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Bool => write!(f, "bool"),
            Self::I64 => write!(f, "i64"),
            Self::I128 => write!(f, "i128"),
            Self::U64 => write!(f, "u64"),
            Self::U128 => write!(f, "u128"),
            Self::F64 => write!(f, "f64"),
            Self::String => write!(f, "string"),
            Self::Array => write!(f, "array"),
            Self::Object => write!(f, "object"),
            Self::Extended(ty) => write!(f, "{ty}"),
            #[cfg(feature = "custom-types")]
            Self::Custom(name) => write!(f, "{name}"),
        }
    }
}

impl Default for ValueType {
    fn default() -> Self {
        Self::Null
    }
}

/// A Value that can be serialized and written
pub trait Writable {
    /// Encodes the value into it's JSON representation as a string
    #[must_use]
    fn encode(&self) -> String;

    /// Encodes the value into it's JSON representation as a string (pretty printed)
    #[must_use]
    fn encode_pp(&self) -> String;

    /// Encodes the value into it's JSON representation into a Writer
    ///
    /// # Errors
    ///
    /// Will return `Err` if an IO error is encountered
    fn write<'writer, W>(&self, w: &mut W) -> io::Result<()>
    where
        W: 'writer + Write;

    /// Encodes the value into it's JSON representation into a Writer, pretty printed
    ///
    /// # Errors
    ///
    /// Will return `Err` if an IO error is encountered.
    fn write_pp<'writer, W>(&self, w: &mut W) -> io::Result<()>
    where
        W: 'writer + Write;
}

#[allow(clippy::trait_duplication_in_bounds)] // This is a bug From<()> is counted as duplicate
/// Support of builder methods for traits.
pub trait Builder<'input>:
    Default
    + From<StaticNode>
    + From<i8>
    + From<i16>
    + From<i32>
    + From<i64>
    + From<u8>
    + From<u16>
    + From<u32>
    + From<u64>
    + From<f32>
    + From<f64>
    + From<bool>
    + From<()>
    + From<String>
    + From<&'input str>
    + From<Cow<'input, str>>
{
    /// Returns an empty array with a given capacity
    fn array_with_capacity(capacity: usize) -> Self;
    /// Returns an empty object with a given capacity
    fn object_with_capacity(capacity: usize) -> Self;
    /// Returns an empty array
    #[must_use]
    fn array() -> Self {
        Self::array_with_capacity(0)
    }
    /// Returns an empty object
    #[must_use]
    fn object() -> Self {
        Self::object_with_capacity(0)
    }
    /// Returns anull value
    fn null() -> Self;
}

/// A type error thrown by the `try_*` functions
#[derive(Clone, Debug)]
pub struct TryTypeError {
    /// The expected value type
    pub expected: ValueType,
    /// The actual value type
    pub got: ValueType,
}

impl std::fmt::Display for TryTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Expected type {}, got {}", self.expected, self.got)
    }
}

impl std::error::Error for TryTypeError {}

/// A trait that specifies how to turn the Value `into` it's sub types
pub trait ValueInto: Sized + ValueAccess {
    /// The type for Strings
    type String;

    /// Tries to turn the value into it's string representation
    #[must_use]
    fn into_string(self) -> Option<Self::String>;

    /// Tries to turn the value into it's string representation
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    fn try_into_string(self) -> Result<Self::String, TryTypeError> {
        let vt = self.value_type();
        self.into_string().ok_or(TryTypeError {
            expected: ValueType::String,
            got: vt,
        })
    }

    /// Tries to turn the value into it's array representation
    #[must_use]
    fn into_array(self) -> Option<Self::Array>;

    /// Tries to turn the value into it's array representation
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    fn try_into_array(self) -> Result<Self::Array, TryTypeError> {
        let vt = self.value_type();
        self.into_array().ok_or(TryTypeError {
            expected: ValueType::Array,
            got: vt,
        })
    }

    /// Tries to turn the value into it's object representation
    #[must_use]
    fn into_object(self) -> Option<Self::Object>;

    /// Tries to turn the value into it's object representation
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    fn try_into_object(self) -> Result<Self::Object, TryTypeError> {
        let vt = self.value_type();
        self.into_object().ok_or(TryTypeError {
            expected: ValueType::Object,
            got: vt,
        })
    }
}

/// Trait to allow accessing data inside a Value
pub trait ValueAccess: Sized {
    /// The target for nested lookups
    type Target: ValueAccess;
    /// The type for Objects
    type Key: Hash + Eq;
    /// The array structure
    type Array: Array<Element = Self::Target>;
    /// The object structure
    type Object: Object<Key = Self::Key, Element = Self::Target>;

    /// Gets the type of the current value
    #[must_use]
    fn value_type(&self) -> ValueType;

    /// Tries to represent the value as a bool
    #[must_use]
    fn as_bool(&self) -> Option<bool>;

    /// Tries to represent the value as a bool
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    fn try_as_bool(&self) -> Result<bool, TryTypeError> {
        self.as_bool().ok_or(TryTypeError {
            expected: ValueType::Bool,
            got: self.value_type(),
        })
    }

    /// Tries to represent the value as an i128
    #[inline]
    #[must_use]
    fn as_i128(&self) -> Option<i128> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }

    /// Tries to represent the value as a i128
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    fn try_as_i128(&self) -> Result<i128, TryTypeError> {
        self.as_i128().ok_or(TryTypeError {
            expected: ValueType::I128,
            got: self.value_type(),
        })
    }

    /// Tries to represent the value as an i64
    #[must_use]
    fn as_i64(&self) -> Option<i64>;

    /// Tries to represent the value as an i64
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    fn try_as_i64(&self) -> Result<i64, TryTypeError> {
        self.as_i64().ok_or(TryTypeError {
            expected: ValueType::I64,
            got: self.value_type(),
        })
    }

    /// Tries to represent the value as an i32
    #[inline]
    #[must_use]
    fn as_i32(&self) -> Option<i32> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }

    /// Tries to represent the value as an i32
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    fn try_as_i32(&self) -> Result<i32, TryTypeError> {
        self.as_i32().ok_or(TryTypeError {
            expected: ValueType::Extended(ExtendedValueType::I32),
            got: self.value_type(),
        })
    }

    /// Tries to represent the value as an i16
    #[inline]
    #[must_use]
    fn as_i16(&self) -> Option<i16> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }

    /// Tries to represent the value as an i16
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    fn try_as_i16(&self) -> Result<i16, TryTypeError> {
        self.as_i16().ok_or(TryTypeError {
            expected: ValueType::Extended(ExtendedValueType::I16),
            got: self.value_type(),
        })
    }

    /// Tries to represent the value as an i8
    #[inline]
    #[must_use]
    fn as_i8(&self) -> Option<i8> {
        self.as_i64().and_then(|u| u.try_into().ok())
    }

    /// Tries to represent the value as an i8
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    fn try_as_i8(&self) -> Result<i8, TryTypeError> {
        self.as_i8().ok_or(TryTypeError {
            expected: ValueType::Extended(ExtendedValueType::I8),
            got: self.value_type(),
        })
    }

    /// Tries to represent the value as an u128
    #[inline]
    #[must_use]
    fn as_u128(&self) -> Option<u128> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }

    /// Tries to represent the value as an u128
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    fn try_as_u128(&self) -> Result<u128, TryTypeError> {
        self.as_u128().ok_or(TryTypeError {
            expected: ValueType::U128,
            got: self.value_type(),
        })
    }

    /// Tries to represent the value as an u64
    #[must_use]
    fn as_u64(&self) -> Option<u64>;

    /// Tries to represent the value as an u64
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    fn try_as_u64(&self) -> Result<u64, TryTypeError> {
        self.as_u64().ok_or(TryTypeError {
            expected: ValueType::U64,
            got: self.value_type(),
        })
    }

    /// Tries to represent the value as an usize
    #[inline]
    #[must_use]
    fn as_usize(&self) -> Option<usize> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }

    /// Tries to represent the value as an usize
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    fn try_as_usize(&self) -> Result<usize, TryTypeError> {
        self.as_usize().ok_or(TryTypeError {
            expected: ValueType::Extended(ExtendedValueType::Usize),
            got: self.value_type(),
        })
    }

    /// Tries to represent the value as an u32
    #[inline]
    #[must_use]
    fn as_u32(&self) -> Option<u32> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }

    /// Tries to represent the value as an u32
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    fn try_as_u32(&self) -> Result<u32, TryTypeError> {
        self.as_u32().ok_or(TryTypeError {
            expected: ValueType::Extended(ExtendedValueType::U32),
            got: self.value_type(),
        })
    }

    /// Tries to represent the value as an u16
    #[inline]
    #[must_use]
    fn as_u16(&self) -> Option<u16> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }

    /// Tries to represent the value as an u16
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    fn try_as_u16(&self) -> Result<u16, TryTypeError> {
        self.as_u16().ok_or(TryTypeError {
            expected: ValueType::Extended(ExtendedValueType::U16),
            got: self.value_type(),
        })
    }

    /// Tries to represent the value as an u8
    #[inline]
    #[must_use]
    fn as_u8(&self) -> Option<u8> {
        self.as_u64().and_then(|u| u.try_into().ok())
    }

    /// Tries to represent the value as an u8
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    fn try_as_u8(&self) -> Result<u8, TryTypeError> {
        self.as_u8().ok_or(TryTypeError {
            expected: ValueType::Extended(ExtendedValueType::U8),
            got: self.value_type(),
        })
    }

    /// Tries to represent the value as a f64
    #[must_use]
    fn as_f64(&self) -> Option<f64>;

    /// Tries to represent the value as a f64
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    fn try_as_f64(&self) -> Result<f64, TryTypeError> {
        self.as_f64().ok_or(TryTypeError {
            expected: ValueType::F64,
            got: self.value_type(),
        })
    }

    /// Casts the current value to a f64 if possible, this will turn integer
    /// values into floats.
    #[must_use]
    #[inline]
    #[allow(clippy::cast_precision_loss, clippy::option_if_let_else)]
    fn cast_f64(&self) -> Option<f64> {
        if let Some(f) = self.as_f64() {
            Some(f)
        } else if let Some(u) = self.as_u128() {
            Some(u as f64)
        } else {
            self.as_i128().map(|i| i as f64)
        }
    }
    /// Tries to Casts the current value to a f64 if possible, this will turn integer
    /// values into floats and error if it isn't possible
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    #[allow(clippy::cast_precision_loss, clippy::option_if_let_else)]
    fn try_cast_f64(&self) -> Result<f64, TryTypeError> {
        if let Some(f) = self.as_f64() {
            Ok(f)
        } else if let Some(u) = self.as_u128() {
            Ok(u as f64)
        } else {
            self.try_as_i128().map(|i| i as f64)
        }
    }

    /// Tries to represent the value as a f32
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    #[must_use]
    fn as_f32(&self) -> Option<f32> {
        self.as_f64().and_then(|u| {
            if u <= f64::from(std::f32::MAX) && u >= f64::from(std::f32::MIN) {
                // Since we check above
                Some(u as f32)
            } else {
                None
            }
        })
    }
    /// Tries to represent the value as a f32
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    fn try_as_f32(&self) -> Result<f32, TryTypeError> {
        self.as_f32().ok_or(TryTypeError {
            expected: ValueType::Extended(ExtendedValueType::F32),
            got: self.value_type(),
        })
    }

    /// Tries to represent the value as a &str
    #[must_use]
    fn as_str(&self) -> Option<&str>;

    /// Tries to represent the value as a &str
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    fn try_as_str(&self) -> Result<&str, TryTypeError> {
        self.as_str().ok_or(TryTypeError {
            expected: ValueType::String,
            got: self.value_type(),
        })
    }

    /// Tries to represent the value as a Char
    #[inline]
    #[must_use]
    fn as_char(&self) -> Option<char> {
        self.as_str().and_then(|s| s.chars().next())
    }

    /// Tries to represent the value as a Char
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    fn try_as_char(&self) -> Result<char, TryTypeError> {
        self.as_char().ok_or(TryTypeError {
            expected: ValueType::Extended(ExtendedValueType::Char),
            got: self.value_type(),
        })
    }

    /// Tries to represent the value as an array and returns a refference to it
    #[must_use]
    fn as_array(&self) -> Option<&Self::Array>;

    /// Tries to represent the value as an array and returns a refference to it
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    fn try_as_array(&self) -> Result<&Self::Array, TryTypeError> {
        self.as_array().ok_or(TryTypeError {
            expected: ValueType::Array,
            got: self.value_type(),
        })
    }

    /// Tries to represent the value as an object and returns a refference to it
    #[must_use]
    fn as_object(&self) -> Option<&Self::Object>;

    /// Tries to represent the value as an object and returns a refference to it
    /// # Errors
    /// if the requested type doesn't match the actual type
    #[inline]
    fn try_as_object(&self) -> Result<&Self::Object, TryTypeError> {
        self.as_object().ok_or(TryTypeError {
            expected: ValueType::Object,
            got: self.value_type(),
        })
    }

    /// Gets a ref to a value based on a key, returns `None` if the
    /// current Value isn't an Object or doesn't contain the key
    /// it was asked for.
    #[inline]
    #[must_use]
    fn get<Q: ?Sized>(&self, k: &Q) -> Option<&Self::Target>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.as_object().and_then(|a| a.get(k))
    }

    /// Trys to get a value based on a key, returns a `TryTypeError` if the
    /// current Value isn't an Object, returns `None` if the key isn't in the object
    /// # Errors
    /// if the value is not an object
    #[inline]
    fn try_get<Q: ?Sized>(&self, k: &Q) -> Result<Option<&Self::Target>, TryTypeError>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        Ok(self
            .as_object()
            .ok_or_else(|| TryTypeError {
                expected: ValueType::Object,
                got: self.value_type(),
            })?
            .get(k))
    }

    /// Checks if a Value contains a given key. This will return
    /// flase if Value isn't an object  
    #[inline]
    #[must_use]
    fn contains_key<Q: ?Sized>(&self, k: &Q) -> bool
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.as_object().and_then(|a| a.get(k)).is_some()
    }

    /// Gets a ref to a value based on n index, returns `None` if the
    /// current Value isn't an Array or doesn't contain the index
    /// it was asked for.
    #[inline]
    #[must_use]
    fn get_idx(&self, i: usize) -> Option<&Self::Target> {
        self.as_array().and_then(|a| a.get(i))
    }

    /// Tries to get a value based on n index, returns a type error if the
    /// current value isn't an Array, returns `None` if the index is out of bouds
    /// # Errors
    /// if the requested type doesn't match the actual type or the value is not an object
    #[inline]
    fn try_get_idx(&self, i: usize) -> Result<Option<&Self::Target>, TryTypeError> {
        Ok(self
            .as_array()
            .ok_or_else(|| TryTypeError {
                expected: ValueType::Array,
                got: self.value_type(),
            })?
            .get(i))
    }

    /// Tries to get an element of an object as a bool
    #[inline]
    #[must_use]
    fn get_bool<Q: ?Sized>(&self, k: &Q) -> Option<bool>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.get(k).and_then(ValueAccess::as_bool)
    }

    /// Tries to get an element of an object as a bool, returns
    /// an error if it isn't bool
    /// # Errors
    /// if the requested type doesn't match the actual type or the value is not an object
    #[inline]
    fn try_get_bool<Q: ?Sized>(&self, k: &Q) -> Result<Option<bool>, TryTypeError>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.try_get(k)?.map(ValueAccess::try_as_bool).transpose()
    }

    /// Tries to get an element of an object as a i128
    #[inline]
    #[must_use]
    fn get_i128<Q: ?Sized>(&self, k: &Q) -> Option<i128>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.get(k).and_then(ValueAccess::as_i128)
    }

    /// Tries to get an element of an object as a i128, returns
    /// an error if it isn't i128
    /// # Errors
    /// if the requested type doesn't match the actual type or the value is not an object
    #[inline]
    fn try_get_i128<Q: ?Sized>(&self, k: &Q) -> Result<Option<i128>, TryTypeError>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.try_get(k)?.map(ValueAccess::try_as_i128).transpose()
    }

    /// Tries to get an element of an object as a i64
    #[inline]
    #[must_use]
    fn get_i64<Q: ?Sized>(&self, k: &Q) -> Option<i64>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.get(k).and_then(ValueAccess::as_i64)
    }

    /// Tries to get an element of an object as a i64, returns
    /// an error if it isn't a i64
    /// # Errors
    /// if the requested type doesn't match the actual type or the value is not an object
    #[inline]
    fn try_get_i64<Q: ?Sized>(&self, k: &Q) -> Result<Option<i64>, TryTypeError>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.try_get(k)?.map(ValueAccess::try_as_i64).transpose()
    }

    /// Tries to get an element of an object as a i32
    #[inline]
    #[must_use]
    fn get_i32<Q: ?Sized>(&self, k: &Q) -> Option<i32>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.get(k).and_then(ValueAccess::as_i32)
    }

    /// Tries to get an element of an object as a i32, returns
    /// an error if it isn't a i32
    /// # Errors
    /// if the requested type doesn't match the actual type or the value is not an object
    #[inline]
    fn try_get_i32<Q: ?Sized>(&self, k: &Q) -> Result<Option<i32>, TryTypeError>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.try_get(k)?.map(ValueAccess::try_as_i32).transpose()
    }

    /// Tries to get an element of an object as a i16
    #[inline]
    #[must_use]
    fn get_i16<Q: ?Sized>(&self, k: &Q) -> Option<i16>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.get(k).and_then(ValueAccess::as_i16)
    }
    /// Tries to get an element of an object as a i16, returns
    /// an error if it isn't a i16
    /// # Errors
    /// if the requested type doesn't match the actual type or the value is not an object
    #[inline]
    fn try_get_i16<Q: ?Sized>(&self, k: &Q) -> Result<Option<i16>, TryTypeError>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.try_get(k)?.map(ValueAccess::try_as_i16).transpose()
    }

    /// Tries to get an element of an object as a i8
    #[inline]
    #[must_use]
    fn get_i8<Q: ?Sized>(&self, k: &Q) -> Option<i8>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.get(k).and_then(ValueAccess::as_i8)
    }

    /// Tries to get an element of an object as a i8, returns
    /// an error if it isn't a i8
    /// # Errors
    /// if the requested type doesn't match the actual type or the value is not an object
    #[inline]
    fn try_get_i8<Q: ?Sized>(&self, k: &Q) -> Result<Option<i8>, TryTypeError>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.try_get(k)?.map(ValueAccess::try_as_i8).transpose()
    }

    /// Tries to get an element of an object as a u128
    #[inline]
    #[must_use]
    fn get_u128<Q: ?Sized>(&self, k: &Q) -> Option<u128>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.get(k).and_then(ValueAccess::as_u128)
    }

    /// Tries to get an element of an object as a u128, returns
    /// an error if it isn't a u128
    /// # Errors
    /// if the requested type doesn't match the actual type or the value is not an object
    #[inline]
    fn try_get_u128<Q: ?Sized>(&self, k: &Q) -> Result<Option<u128>, TryTypeError>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.try_get(k)?.map(ValueAccess::try_as_u128).transpose()
    }

    /// Tries to get an element of an object as a u64
    #[inline]
    #[must_use]
    fn get_u64<Q: ?Sized>(&self, k: &Q) -> Option<u64>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.get(k).and_then(ValueAccess::as_u64)
    }

    /// Tries to get an element of an object as a u64, returns
    /// an error if it isn't a u64
    /// # Errors
    /// if the requested type doesn't match the actual type or the value is not an object
    #[inline]
    fn try_get_u64<Q: ?Sized>(&self, k: &Q) -> Result<Option<u64>, TryTypeError>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.try_get(k)?.map(ValueAccess::try_as_u64).transpose()
    }

    /// Tries to get an element of an object as a usize
    #[inline]
    #[must_use]
    fn get_usize<Q: ?Sized>(&self, k: &Q) -> Option<usize>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.get(k).and_then(ValueAccess::as_usize)
    }

    /// Tries to get an element of an object as a usize, returns
    /// an error if it isn't a usize
    /// # Errors
    /// if the requested type doesn't match the actual type or the value is not an object
    #[inline]
    fn try_get_usize<Q: ?Sized>(&self, k: &Q) -> Result<Option<usize>, TryTypeError>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.try_get(k)?.map(ValueAccess::try_as_usize).transpose()
    }

    /// Tries to get an element of an object as a u32
    #[inline]
    #[must_use]
    fn get_u32<Q: ?Sized>(&self, k: &Q) -> Option<u32>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.get(k).and_then(ValueAccess::as_u32)
    }

    /// Tries to get an element of an object as a u32, returns
    /// an error if it isn't a u32
    /// # Errors
    /// if the requested type doesn't match the actual type or the value is not an object
    #[inline]
    fn try_get_u32<Q: ?Sized>(&self, k: &Q) -> Result<Option<u32>, TryTypeError>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.try_get(k)?.map(ValueAccess::try_as_u32).transpose()
    }

    /// Tries to get an element of an object as a u16
    #[inline]
    #[must_use]
    fn get_u16<Q: ?Sized>(&self, k: &Q) -> Option<u16>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.get(k).and_then(ValueAccess::as_u16)
    }

    /// Tries to get an element of an object as a u16, returns
    /// an error if it isn't a u16
    /// # Errors
    /// if the requested type doesn't match the actual type or the value is not an object
    #[inline]
    fn try_get_u16<Q: ?Sized>(&self, k: &Q) -> Result<Option<u16>, TryTypeError>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.try_get(k)?.map(ValueAccess::try_as_u16).transpose()
    }

    /// Tries to get an element of an object as a u8
    #[inline]
    #[must_use]
    fn get_u8<Q: ?Sized>(&self, k: &Q) -> Option<u8>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.get(k).and_then(ValueAccess::as_u8)
    }

    /// Tries to get an element of an object as a u8, returns
    /// an error if it isn't a u8
    /// # Errors
    /// if the requested type doesn't match the actual type or the value is not an object
    #[inline]
    fn try_get_u8<Q: ?Sized>(&self, k: &Q) -> Result<Option<u8>, TryTypeError>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.try_get(k)?.map(ValueAccess::try_as_u8).transpose()
    }

    /// Tries to get an element of an object as a f64
    #[inline]
    #[must_use]
    fn get_f64<Q: ?Sized>(&self, k: &Q) -> Option<f64>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.get(k).and_then(ValueAccess::as_f64)
    }

    /// Tries to get an element of an object as a u8, returns
    /// an error if it isn't a u8
    /// # Errors
    /// if the requested type doesn't match the actual type or the value is not an object
    #[inline]
    fn try_get_f64<Q: ?Sized>(&self, k: &Q) -> Result<Option<f64>, TryTypeError>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.try_get(k)?.map(ValueAccess::try_as_f64).transpose()
    }

    /// Tries to get an element of an object as a f32
    #[inline]
    #[must_use]
    fn get_f32<Q: ?Sized>(&self, k: &Q) -> Option<f32>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.get(k).and_then(ValueAccess::as_f32)
    }

    /// Tries to get an element of an object as a f32, returns
    /// an error if it isn't a f32
    /// # Errors
    /// if the requested type doesn't match the actual type or the value is not an object
    #[inline]
    fn try_get_f32<Q: ?Sized>(&self, k: &Q) -> Result<Option<f32>, TryTypeError>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.try_get(k)?.map(ValueAccess::try_as_f32).transpose()
    }

    /// Tries to get an element of an object as a str
    #[inline]
    #[must_use]
    fn get_str<Q: ?Sized>(&self, k: &Q) -> Option<&str>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.get(k).and_then(ValueAccess::as_str)
    }

    /// Tries to get an element of an object as a str, returns
    /// an error if it isn't a str
    /// # Errors
    /// if the requested type doesn't match the actual type or the value is not an object
    #[inline]
    fn try_get_str<Q: ?Sized>(&self, k: &Q) -> Result<Option<&str>, TryTypeError>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.try_get(k)
            .and_then(|s| s.map(ValueAccess::try_as_str).transpose())
    }

    /// Tries to get an element of an object as a array
    #[inline]
    #[must_use]
    fn get_array<Q: ?Sized>(
        &self,
        k: &Q,
    ) -> Option<&<<Self as ValueAccess>::Target as ValueAccess>::Array>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.get(k).and_then(ValueAccess::as_array)
    }

    /// Tries to get an element of an object as an array, returns
    /// an error if it isn't a array
    /// # Errors
    /// if the requested type doesn't match the actual type or the value is not an object
    #[inline]
    fn try_get_array<Q: ?Sized>(
        &self,
        k: &Q,
    ) -> Result<Option<&<<Self as ValueAccess>::Target as ValueAccess>::Array>, TryTypeError>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.try_get(k)
            .and_then(|s| s.map(ValueAccess::try_as_array).transpose())
    }

    /// Tries to get an element of an object as a object
    #[inline]
    #[must_use]
    fn get_object<Q: ?Sized>(
        &self,
        k: &Q,
    ) -> Option<&<<Self as ValueAccess>::Target as ValueAccess>::Object>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.get(k).and_then(ValueAccess::as_object)
    }

    /// Tries to get an element of an object as an object, returns
    /// an error if it isn't an object
    ///
    /// # Errors
    /// if the requested type doesn't match the actual type or the value is not an object
    #[inline]
    fn try_get_object<Q: ?Sized>(
        &self,
        k: &Q,
    ) -> Result<Option<&<<Self as ValueAccess>::Target as ValueAccess>::Object>, TryTypeError>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.try_get(k)
            .and_then(|s| s.map(ValueAccess::try_as_object).transpose())
    }
}
/// The `Value` exposes common interface for values, this allows using both
/// `BorrowedValue` and `OwnedValue` nearly interchangable
pub trait Value:
    Sized
    + Index<usize>
    + PartialEq<i8>
    + PartialEq<i16>
    + PartialEq<i32>
    + PartialEq<i64>
    + PartialEq<i128>
    + PartialEq<u8>
    + PartialEq<u16>
    + PartialEq<u32>
    + PartialEq<u64>
    + PartialEq<u128>
    + PartialEq<f32>
    + PartialEq<f64>
    + PartialEq<String>
    + PartialEq<bool>
    + PartialEq<()>
    + ValueAccess
{
    /// returns true if the current value is null
    #[must_use]
    fn is_null(&self) -> bool;

    /// returns true if the current value a floatingpoint number
    #[inline]
    #[must_use]
    fn is_float(&self) -> bool {
        self.is_f64()
    }

    /// returns true if the current value a integer number
    #[inline]
    #[must_use]
    fn is_integer(&self) -> bool {
        self.is_i128() || self.is_u128()
    }

    /// returns true if the current value a number either float or integer
    #[inline]
    #[must_use]
    fn is_number(&self) -> bool {
        self.is_float() || self.is_integer()
    }

    /// returns true if the current value a bool
    #[inline]
    #[must_use]
    fn is_bool(&self) -> bool {
        self.as_bool().is_some()
    }

    /// returns true if the current value can be represented as a i128
    #[inline]
    #[must_use]
    fn is_i128(&self) -> bool {
        self.as_i128().is_some()
    }

    /// returns true if the current value can be represented as a i64
    #[inline]
    #[must_use]
    fn is_i64(&self) -> bool {
        self.as_i64().is_some()
    }

    /// returns true if the current value can be represented as a i32
    #[inline]
    #[must_use]
    fn is_i32(&self) -> bool {
        self.as_i32().is_some()
    }

    /// returns true if the current value can be represented as a i16
    #[inline]
    #[must_use]
    fn is_i16(&self) -> bool {
        self.as_i16().is_some()
    }

    /// returns true if the current value can be represented as a i8
    #[inline]
    #[must_use]
    fn is_i8(&self) -> bool {
        self.as_i8().is_some()
    }

    /// returns true if the current value can be represented as a u128
    #[inline]
    #[must_use]
    fn is_u128(&self) -> bool {
        self.as_u128().is_some()
    }

    /// returns true if the current value can be represented as a u64
    #[inline]
    #[must_use]
    fn is_u64(&self) -> bool {
        self.as_u64().is_some()
    }

    /// returns true if the current value can be represented as a usize
    #[inline]
    #[must_use]
    fn is_usize(&self) -> bool {
        self.as_usize().is_some()
    }

    /// returns true if the current value can be represented as a u32
    #[inline]
    #[must_use]
    fn is_u32(&self) -> bool {
        self.as_u32().is_some()
    }

    /// returns true if the current value can be represented as a u16
    #[inline]
    #[must_use]
    fn is_u16(&self) -> bool {
        self.as_u16().is_some()
    }

    /// returns true if the current value can be represented as a u8
    #[inline]
    #[must_use]
    fn is_u8(&self) -> bool {
        self.as_u8().is_some()
    }

    /// returns true if the current value can be represented as a f64
    #[inline]
    #[must_use]
    fn is_f64(&self) -> bool {
        self.as_f64().is_some()
    }

    /// returns true if the current value can be cast into a f64
    #[inline]
    #[must_use]
    fn is_f64_castable(&self) -> bool {
        self.cast_f64().is_some()
    }

    /// returns true if the current value can be represented as a f64
    #[inline]
    #[must_use]
    fn is_f32(&self) -> bool {
        self.as_f32().is_some()
    }

    /// returns true if the current value can be represented as a str
    #[inline]
    #[must_use]
    fn is_str(&self) -> bool {
        self.as_str().is_some()
    }

    /// returns true if the current value can be represented as a char
    #[inline]
    #[must_use]
    fn is_char(&self) -> bool {
        self.as_char().is_some()
    }

    /// returns true if the current value can be represented as an array
    #[inline]
    #[must_use]
    fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    /// returns true if the current value can be represented as an object
    #[inline]
    #[must_use]
    fn is_object(&self) -> bool {
        self.as_object().is_some()
    }

    #[cfg(feature = "custom-types")]
    /// returns if a type is a custom type
    fn is_custom(&self) -> bool {
        false
    }
}

/// Mutatability for values
pub trait Mutable: IndexMut<usize> + Value + Sized {
    /// Insert into this `Value` as an `Object`.
    /// Will return an `AccessError::NotAnObject` if called
    /// on a `Value` that isn't an object - otherwise will
    /// behave the same as `HashMap::insert`
    /// # Errors
    ///
    /// Will return `Err` if `self` is not an object.
    #[inline]
    fn insert<K, V>(&mut self, k: K, v: V) -> std::result::Result<Option<Self::Target>, AccessError>
    where
        K: Into<<Self as ValueAccess>::Key>,
        V: Into<<Self as ValueAccess>::Target>,
        <Self as ValueAccess>::Key: Hash + Eq,
    {
        self.as_object_mut()
            .ok_or(AccessError::NotAnObject)
            .map(|o| o.insert(k.into(), v.into()))
    }

    /// Tries to insert into this `Value` as an `Object`.
    /// If the `Value` isn't an object this opoeration will
    /// return `None` and have no effect.
    #[inline]
    fn try_insert<K, V>(&mut self, k: K, v: V) -> Option<Self::Target>
    where
        K: Into<<Self as ValueAccess>::Key>,
        V: Into<<Self as ValueAccess>::Target>,
        <Self as ValueAccess>::Key: Hash + Eq,
    {
        self.insert(k, v).ok().flatten()
    }

    /// Remove from this `Value` as an `Object`.
    /// Will return an `AccessError::NotAnObject` if called
    /// on a `Value` that isn't an object - otherwise will
    /// behave the same as `HashMap::remove`
    /// # Errors
    ///
    /// Will return `Err` if `self` is not an Object.
    #[inline]
    fn remove<Q: ?Sized>(&mut self, k: &Q) -> std::result::Result<Option<Self::Target>, AccessError>
    where
        <Self as ValueAccess>::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.as_object_mut()
            .ok_or(AccessError::NotAnObject)
            .map(|o| o.remove(k))
    }

    /// Tries to remove from this `Value` as an `Object`.
    /// If the `Value` isn't an object this opoeration will
    /// return `None` and have no effect.
    #[inline]
    fn try_remove<Q: ?Sized>(&mut self, k: &Q) -> Option<Self::Target>
    where
        <Self as ValueAccess>::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.remove(k).ok().flatten()
    }

    /// Pushes to this `Value` as an `Array`.
    /// Will return an `AccessError::NotAnArray` if called
    /// on a `Value` that isn't an `Array` - otherwise will
    /// behave the same as `Vec::push`
    /// # Errors
    ///
    /// Will return `Err` if `self` is not an array.
    #[inline]
    fn push<V>(&mut self, v: V) -> std::result::Result<(), AccessError>
    where
        V: Into<<Self as ValueAccess>::Target>,
    {
        self.as_array_mut()
            .ok_or(AccessError::NotAnArray)
            .map(|o| o.push(v.into()))
    }

    /// Tries to push to a `Value` if as an `Array`.
    /// This funciton will have no effect if `Value` is of
    /// a different type
    fn try_push<V>(&mut self, v: V)
    where
        V: Into<<Self as ValueAccess>::Target>,
    {
        let _: Result<_, _> = self.push(v);
    }

    /// Pops from this `Value` as an `Array`.
    /// Will return an `AccessError::NotAnArray` if called
    /// on a `Value` that isn't an `Array` - otherwise will
    /// behave the same as `Vec::pop`
    /// # Errors
    ///
    /// Will return `Err` if `self` is not an array.
    #[inline]
    fn pop(&mut self) -> std::result::Result<Option<Self::Target>, AccessError> {
        self.as_array_mut()
            .ok_or(AccessError::NotAnArray)
            .map(Array::pop)
    }

    /// Tries to pop from a `Value` as an `Array`.
    /// if the `Value` is any other type `None` will
    /// always be returned
    #[inline]
    fn try_pop(&mut self) -> Option<Self::Target> {
        self.pop().ok().flatten()
    }

    /// Same as `get` but returns a mutable ref instead
    //    fn get_amut(&mut self, k: &str) -> Option<&mut Self>;
    fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut Self::Target>
    where
        <Self as ValueAccess>::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        self.as_object_mut().and_then(|m| m.get_mut(k))
    }

    /// Same as `get_idx` but returns a mutable ref instead
    #[inline]
    fn get_idx_mut(&mut self, i: usize) -> Option<&mut Self::Target> {
        self.as_array_mut().and_then(|a| a.get_mut(i))
    }
    /// Tries to represent the value as an array and returns a mutable refference to it
    fn as_array_mut(&mut self) -> Option<&mut <Self as ValueAccess>::Array>;
    /// Tries to represent the value as an object and returns a mutable refference to it
    fn as_object_mut(&mut self) -> Option<&mut Self::Object>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
