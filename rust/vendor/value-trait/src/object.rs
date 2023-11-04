#[cfg(feature = "halfbrown")]
use halfbrown::HashMap as Halfbrown;
#[cfg(feature = "hashbrown")]
use hashbrown::HashMap as Hashbrown;
use std::collections::HashMap;
use std::hash::Hash;
use std::{borrow::Borrow, hash::BuildHasher};

/// A JSON Object
pub trait Object {
    /// The key in the objects
    type Key;
    /// The values in the object
    type Element;

    /// Gets a ref to a value based on a key, returns `None` if the
    /// current Value isn't an Object or doesn't contain the key
    /// it was asked for.
    #[must_use]
    fn get<Q: ?Sized>(&self, k: &Q) -> Option<&Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord;

    /// Gets the value of a key as a mutable reference.
    #[must_use]
    fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord;

    /// Inserts a value
    #[must_use]
    fn insert<K, V>(&mut self, k: K, v: V) -> Option<Self::Element>
    where
        K: Into<Self::Key>,
        V: Into<Self::Element>,
        Self::Key: Hash + Eq;

    /// Removes a value from the object
    #[must_use]
    fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<Self::Element>
    where
        Self::Key: Borrow<Q>,
        Q: Hash + Eq + Ord;

    /// Iterates over the key value paris
    #[must_use]
    fn iter<'i>(&'i self) -> Box<dyn Iterator<Item = (&Self::Key, &Self::Element)> + 'i>;

    /// Iterates over the keys
    #[must_use]
    fn keys<'i>(&'i self) -> Box<dyn Iterator<Item = &Self::Key> + 'i>;

    /// Iterates over the values
    #[must_use]
    fn values<'i>(&'i self) -> Box<dyn Iterator<Item = &Self::Element> + 'i>;

    /// Number of key/value pairs
    #[must_use]
    fn len(&self) -> usize;

    /// Returns if the array is empty
    #[must_use]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(feature = "halfbrown")]
impl<MapK, MapE, S> Object for Halfbrown<MapK, MapE, S>
where
    MapK: Hash + Eq,
    S: BuildHasher + Default,
{
    type Key = MapK;
    type Element = MapE;

    #[inline]
    fn get<Q: ?Sized>(&self, k: &Q) -> Option<&Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        Halfbrown::get(self, k)
    }

    #[inline]
    fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        Halfbrown::get_mut(self, k)
    }

    #[inline]
    fn insert<K, V>(&mut self, k: K, v: V) -> Option<Self::Element>
    where
        K: Into<Self::Key>,
        V: Into<Self::Element>,
        Self::Key: Hash + Eq,
    {
        Halfbrown::insert(self, k.into(), v.into())
    }

    #[inline]
    fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        Halfbrown::remove(self, k)
    }

    #[inline]
    fn iter<'i>(&'i self) -> Box<dyn Iterator<Item = (&Self::Key, &Self::Element)> + 'i> {
        Box::new(Halfbrown::iter(self))
    }

    #[inline]
    fn keys<'i>(&'i self) -> Box<dyn Iterator<Item = &Self::Key> + 'i> {
        Box::new(Halfbrown::keys(self))
    }

    #[inline]
    fn values<'i>(&'i self) -> Box<dyn Iterator<Item = &Self::Element> + 'i> {
        Box::new(Halfbrown::values(self))
    }

    #[inline]
    fn len(&self) -> usize {
        Halfbrown::len(self)
    }
}

impl<MapK, MapE, S: BuildHasher> Object for HashMap<MapK, MapE, S>
where
    MapK: Hash + Eq,
{
    type Key = MapK;
    type Element = MapE;

    #[inline]
    fn get<Q: ?Sized>(&self, k: &Q) -> Option<&Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        HashMap::get(self, k)
    }

    #[inline]
    fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        HashMap::get_mut(self, k)
    }

    #[inline]
    fn insert<K, V>(&mut self, k: K, v: V) -> Option<Self::Element>
    where
        K: Into<Self::Key>,
        V: Into<Self::Element>,
        Self::Key: Hash + Eq,
    {
        HashMap::insert(self, k.into(), v.into())
    }

    #[inline]
    fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        HashMap::remove(self, k)
    }

    #[inline]
    fn iter<'i>(&'i self) -> Box<dyn Iterator<Item = (&Self::Key, &Self::Element)> + 'i> {
        Box::new(HashMap::iter(self))
    }

    #[inline]
    fn keys<'i>(&'i self) -> Box<dyn Iterator<Item = &Self::Key> + 'i> {
        Box::new(HashMap::keys(self))
    }

    #[inline]
    fn values<'i>(&'i self) -> Box<dyn Iterator<Item = &Self::Element> + 'i> {
        Box::new(HashMap::values(self))
    }

    #[inline]
    fn len(&self) -> usize {
        HashMap::len(self)
    }
}

#[cfg(feature = "hashbrown")]
impl<MapK, MapE, S: BuildHasher> Object for Hashbrown<MapK, MapE, S>
where
    MapK: Hash + Eq,
{
    type Key = MapK;
    type Element = MapE;

    #[inline]
    fn get<Q: ?Sized>(&self, k: &Q) -> Option<&Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        Hashbrown::get(self, k)
    }

    #[inline]
    fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        Hashbrown::get_mut(self, k)
    }

    #[inline]
    fn insert<K, V>(&mut self, k: K, v: V) -> Option<Self::Element>
    where
        K: Into<Self::Key>,
        V: Into<Self::Element>,
        Self::Key: Hash + Eq,
    {
        Hashbrown::insert(self, k.into(), v.into())
    }

    #[inline]
    fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        Hashbrown::remove(self, k)
    }

    #[inline]
    fn iter<'i>(&'i self) -> Box<dyn Iterator<Item = (&Self::Key, &Self::Element)> + 'i> {
        Box::new(Hashbrown::iter(self))
    }

    #[inline]
    fn keys<'i>(&'i self) -> Box<dyn Iterator<Item = &Self::Key> + 'i> {
        Box::new(Hashbrown::keys(self))
    }

    #[inline]
    fn values<'i>(&'i self) -> Box<dyn Iterator<Item = &Self::Element> + 'i> {
        Box::new(Hashbrown::values(self))
    }

    #[inline]
    fn len(&self) -> usize {
        Hashbrown::len(self)
    }
}

#[cfg(feature = "c-abi")]
impl<MapK, MapE, S: ::std::hash::BuildHasher> Object
    for abi_stable::std_types::RHashMap<MapK, MapE, S>
where
    MapK: Hash + Eq,
{
    type Key = MapK;
    type Element = MapE;

    #[inline]
    fn get<Q: ?Sized>(&self, k: &Q) -> Option<&Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        abi_stable::std_types::RHashMap::get(self, k)
    }

    #[inline]
    fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        abi_stable::std_types::RHashMap::get_mut(self, k)
    }

    #[inline]
    fn insert<K, V>(&mut self, k: K, v: V) -> Option<Self::Element>
    where
        K: Into<Self::Key>,
        V: Into<Self::Element>,
        Self::Key: Hash + Eq,
    {
        abi_stable::std_types::RHashMap::insert(self, k.into(), v.into()).into()
    }

    #[inline]
    fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<Self::Element>
    where
        Self::Key: Borrow<Q> + Hash + Eq,
        Q: Hash + Eq + Ord,
    {
        abi_stable::std_types::RHashMap::remove(self, k).into()
    }

    #[inline]
    fn iter<'i>(&'i self) -> Box<dyn Iterator<Item = (&Self::Key, &Self::Element)> + 'i> {
        Box::new(abi_stable::std_types::RHashMap::iter(self).map(Into::into))
    }

    #[inline]
    fn keys<'i>(&'i self) -> Box<dyn Iterator<Item = &Self::Key> + 'i> {
        Box::new(abi_stable::std_types::RHashMap::keys(self))
    }

    #[inline]
    fn values<'i>(&'i self) -> Box<dyn Iterator<Item = &Self::Element> + 'i> {
        Box::new(abi_stable::std_types::RHashMap::values(self))
    }

    #[inline]
    fn len(&self) -> usize {
        abi_stable::std_types::RHashMap::len(self)
    }
}
