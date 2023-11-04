use crate::Value;
use std::ops::Index;
use std::slice::SliceIndex;

/// Functions guaranteed for any array object
pub trait Array: Index<usize> + Sync + Send + Clone {
    /// Elements of the array
    type Element: Value;

    /// Gets a ref to a value based on n index, returns `None` if the
    /// current Value isn't an Array or doesn't contain the index
    /// it was asked for.
    #[must_use]
    fn get<I>(&self, i: I) -> Option<&<I as SliceIndex<[Self::Element]>>::Output>
    where
        I: SliceIndex<[Self::Element]>;

    /// Gets a ref to a value based on n index, returns `None` if the
    /// current Value isn't an Array or doesn't contain the index
    /// it was asked for.
    #[must_use]
    fn get_mut(&mut self, i: usize) -> Option<&mut Self::Element>;

    /// Returns the last element of the array or `None`
    #[must_use]
    fn pop(&mut self) -> Option<Self::Element>;

    /// Appends e to the end of the `Array`
    fn push(&mut self, e: Self::Element);

    /// Iterates over the values paris
    #[must_use]
    fn iter<'i>(&'i self) -> Box<dyn Iterator<Item = &Self::Element> + 'i>;

    /// Number of key/value pairs
    #[must_use]
    fn len(&self) -> usize;

    /// Returns if the array is empty
    #[must_use]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Array for Vec<T>
where
    T: Value + Sync + Send + Clone,
{
    type Element = T;
    #[inline]
    fn get<I>(&self, i: I) -> Option<&<I as SliceIndex<[T]>>::Output>
    where
        I: SliceIndex<[T]>,
    {
        <[T]>::get(self, i)
    }
    #[inline]
    fn get_mut(&mut self, i: usize) -> Option<&mut T> {
        <[T]>::get_mut(self, i)
    }

    #[inline]
    fn pop(&mut self) -> Option<T> {
        Vec::pop(self)
    }

    #[inline]
    fn push(&mut self, e: T) {
        Vec::push(self, e);
    }

    fn iter<'i>(&'i self) -> Box<dyn Iterator<Item = &T> + 'i> {
        Box::new(<[T]>::iter(self))
    }

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }
}

#[cfg(feature = "c-abi")]
impl<T> Array for abi_stable::std_types::RVec<T>
where
    T: Value + Sync + Send + Clone,
{
    type Element = T;

    #[inline]
    fn get<I>(&self, i: I) -> Option<&<I as SliceIndex<[T]>>::Output>
    where
        I: SliceIndex<[T]>,
    {
        <[T]>::get(self, i)
    }

    #[inline]
    fn get_mut(&mut self, i: usize) -> Option<&mut T> {
        <[T]>::get_mut(self, i)
    }

    #[inline]
    fn pop(&mut self) -> Option<T> {
        abi_stable::std_types::RVec::pop(self)
    }

    #[inline]
    fn push(&mut self, e: T) {
        abi_stable::std_types::RVec::push(self, e);
    }

    fn iter<'i>(&'i self) -> Box<dyn Iterator<Item = &T> + 'i> {
        Box::new(<[T]>::iter(self))
    }

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }
}
