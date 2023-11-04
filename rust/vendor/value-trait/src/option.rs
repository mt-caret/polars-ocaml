use crate::{ExtendedValueType, ValueAccess, ValueInto, ValueType};

impl<V> ValueInto for Option<V>
where
    V: ValueInto,
{
    type String = V::String;

    fn into_string(self) -> Option<Self::String> {
        self.and_then(ValueInto::into_string)
    }
    fn into_array(self) -> Option<Self::Array> {
        self.and_then(ValueInto::into_array)
    }
    fn into_object(self) -> Option<Self::Object> {
        self.and_then(ValueInto::into_object)
    }
}
impl<V> ValueAccess for Option<V>
where
    V: ValueAccess,
{
    type Target = V::Target;
    type Key = V::Key;
    type Array = V::Array;
    type Object = V::Object;

    fn value_type(&self) -> ValueType {
        self.as_ref().map_or(
            ValueType::Extended(ExtendedValueType::None),
            ValueAccess::value_type,
        )
    }
    fn as_bool(&self) -> Option<bool> {
        self.as_ref().and_then(ValueAccess::as_bool)
    }

    fn as_i64(&self) -> Option<i64> {
        self.as_ref().and_then(ValueAccess::as_i64)
    }

    fn as_u64(&self) -> Option<u64> {
        self.as_ref().and_then(ValueAccess::as_u64)
    }

    fn as_f64(&self) -> Option<f64> {
        self.as_ref().and_then(ValueAccess::as_f64)
    }

    fn as_str(&self) -> Option<&str> {
        self.as_ref().and_then(ValueAccess::as_str)
    }

    fn as_array(&self) -> Option<&Self::Array> {
        self.as_ref().and_then(ValueAccess::as_array)
    }

    fn as_object(&self) -> Option<&Self::Object> {
        self.as_ref().and_then(ValueAccess::as_object)
    }
}

impl<V, E> ValueInto for Result<V, E>
where
    V: ValueInto,
{
    type String = V::String;

    fn into_string(self) -> Option<Self::String> {
        self.ok().and_then(ValueInto::into_string)
    }
    fn into_array(self) -> Option<Self::Array> {
        self.ok().and_then(ValueInto::into_array)
    }
    fn into_object(self) -> Option<Self::Object> {
        self.ok().and_then(ValueInto::into_object)
    }
}

impl<V, E> ValueAccess for Result<V, E>
where
    V: ValueAccess,
{
    type Target = V::Target;
    type Key = V::Key;
    type Array = V::Array;
    type Object = V::Object;

    fn value_type(&self) -> ValueType {
        self.as_ref().ok().map_or(
            ValueType::Extended(ExtendedValueType::None),
            ValueAccess::value_type,
        )
    }

    fn as_bool(&self) -> Option<bool> {
        self.as_ref().ok().and_then(ValueAccess::as_bool)
    }

    fn as_i64(&self) -> Option<i64> {
        self.as_ref().ok().and_then(ValueAccess::as_i64)
    }

    fn as_u64(&self) -> Option<u64> {
        self.as_ref().ok().and_then(ValueAccess::as_u64)
    }

    fn as_f64(&self) -> Option<f64> {
        self.as_ref().ok().and_then(ValueAccess::as_f64)
    }

    fn as_str(&self) -> Option<&str> {
        self.as_ref().ok().and_then(ValueAccess::as_str)
    }

    fn as_array(&self) -> Option<&Self::Array> {
        self.as_ref().ok().and_then(ValueAccess::as_array)
    }

    fn as_object(&self) -> Option<&Self::Object> {
        self.as_ref().ok().and_then(ValueAccess::as_object)
    }
}

impl<V> ValueAccess for &V
where
    V: ValueAccess,
{
    type Target = V::Target;
    type Key = V::Key;
    type Array = V::Array;
    type Object = V::Object;

    fn value_type(&self) -> ValueType {
        (*self).value_type()
    }
    fn as_bool(&self) -> Option<bool> {
        (*self).as_bool()
    }

    fn as_i64(&self) -> Option<i64> {
        (*self).as_i64()
    }

    fn as_u64(&self) -> Option<u64> {
        (*self).as_u64()
    }

    fn as_f64(&self) -> Option<f64> {
        (*self).as_f64()
    }

    fn as_str(&self) -> Option<&str> {
        (*self).as_str()
    }

    fn as_array(&self) -> Option<&Self::Array> {
        (*self).as_array()
    }

    fn as_object(&self) -> Option<&Self::Object> {
        (*self).as_object()
    }
}
