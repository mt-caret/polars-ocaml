use crate::{base::ValueAsScalar, derived::TypedScalarValue};

use super::StaticNode;

impl PartialEq<()> for StaticNode {
    #[inline]
    #[must_use]
    fn eq(&self, _other: &()) -> bool {
        self.is_null()
    }
}

impl PartialEq<bool> for StaticNode {
    #[inline]
    #[must_use]
    fn eq(&self, other: &bool) -> bool {
        self.as_bool().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<str> for StaticNode {
    #[inline]
    #[must_use]
    fn eq(&self, other: &str) -> bool {
        self.as_str().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<&str> for StaticNode {
    #[inline]
    #[must_use]
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl PartialEq<String> for StaticNode {
    #[inline]
    #[must_use]
    fn eq(&self, other: &String) -> bool {
        self.as_str().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<i8> for StaticNode {
    #[inline]
    #[must_use]
    fn eq(&self, other: &i8) -> bool {
        self.as_i8().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<i16> for StaticNode {
    #[inline]
    #[must_use]
    fn eq(&self, other: &i16) -> bool {
        self.as_i16().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<i32> for StaticNode {
    #[inline]
    #[must_use]
    fn eq(&self, other: &i32) -> bool {
        self.as_i32().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<i64> for StaticNode {
    #[inline]
    #[must_use]
    fn eq(&self, other: &i64) -> bool {
        self.as_i64().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<i128> for StaticNode {
    #[inline]
    #[must_use]
    fn eq(&self, other: &i128) -> bool {
        self.as_i128().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<u8> for StaticNode {
    #[inline]
    #[must_use]
    fn eq(&self, other: &u8) -> bool {
        self.as_u8().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<u16> for StaticNode {
    #[inline]
    #[must_use]
    fn eq(&self, other: &u16) -> bool {
        self.as_u16().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<u32> for StaticNode {
    #[inline]
    #[must_use]
    fn eq(&self, other: &u32) -> bool {
        self.as_u32().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<u64> for StaticNode {
    #[inline]
    #[must_use]
    fn eq(&self, other: &u64) -> bool {
        self.as_u64().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<usize> for StaticNode {
    #[inline]
    #[must_use]
    fn eq(&self, other: &usize) -> bool {
        self.as_usize().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<u128> for StaticNode {
    #[inline]
    #[must_use]
    fn eq(&self, other: &u128) -> bool {
        self.as_u128().map(|t| t.eq(other)).unwrap_or_default()
    }
}

impl PartialEq<f32> for StaticNode {
    #[inline]
    #[must_use]
    fn eq(&self, other: &f32) -> bool {
        self.as_f32().map(|t| t.eq(other)).unwrap_or_default()
    }
}
impl PartialEq<f64> for StaticNode {
    #[inline]
    #[must_use]
    fn eq(&self, other: &f64) -> bool {
        self.as_f64().map(|t| t.eq(other)).unwrap_or_default()
    }
}
