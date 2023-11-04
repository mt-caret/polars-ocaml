//! A [`std::hash::Hasher`](https://doc.rust-lang.org/std/hash/trait.Hasher.html) which is designed
//! to work with already-hashed or hash-like data.
//!
//! The provided hasher does minimal work under the assumption that the input data is already
//! suitable for use as a key in a `HashSet` or `HashMap`.
//!
//! As well as the performance benefit, it also causes `HashSet`s or `HashMap`s to become somewhat
//! deterministic.  Given two equal `HashSet`s or `HashMap`s containing more than a single element,
//! iterating them will yield the elements in differing orders.  By using a
//! [`hash_hasher::HashedSet`](type.HashedSet.html) or
//! [`hash_hasher::HashedMap`](type.HashedMap.html), then if the same data is inserted and/or
//! removed *in the same order*, iterating the collection will yield a consistent order.
//!
//! # Examples
//!
//! Since `new()` and `with_capacity()` aren't available for `HashSet`s or `HashMap`s using custom
//! hashers, the available constructors are `default()`, `with_hasher()` and
//! `with_capacity_and_hasher()`.
//!
//! ## Using `default()`
//!
//! ```
//! use hash_hasher::HashedMap;
//!
//! let mut map = HashedMap::default();
//! assert!(map.insert(0, "zero").is_none());
//! ```
//!
//! ## Using `with_capacity_and_hasher()`
//!
//! ```
//! use hash_hasher::{HashBuildHasher, HashedSet};
//!
//! let mut set = HashedSet::with_capacity_and_hasher(100, HashBuildHasher::default());
//! assert!(set.insert(0));
//! ```

#![doc(test(attr(forbid(warnings))))]
#![warn(unused, missing_copy_implementations, missing_docs)]
#![deny(
    deprecated_in_future,
    future_incompatible,
    macro_use_extern_crate,
    rust_2018_idioms,
    nonstandard_style,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    warnings,
    clippy::all,
    clippy::pedantic
)]
#![forbid(
    const_err,
    invalid_type_param_default,
    macro_expanded_macro_exports_accessed_by_absolute_paths,
    missing_fragment_specifier,
    mutable_transmutes,
    no_mangle_const_items,
    order_dependent_trait_objects,
    overflowing_literals,
    pub_use_of_private_extern_crate,
    unknown_crate_types
)]

use std::hash::{BuildHasherDefault, Hasher};

/// A hasher which does minimal work to create the required `u64` output under the assumption that
/// the input is already a hash digest or otherwise already suitable for use as a key in a `HashSet`
/// or `HashMap`.
#[derive(Copy, Clone, Default)]
pub struct HashHasher(u64);

impl Hasher for HashHasher {
    #[inline]
    fn write(&mut self, value: &[u8]) {
        // A normal use-case (e.g. by a node in a DHT) may well involve handling hashes which are
        // identical over the most-significant-bits, hence reverse the input message here to use the
        // least-significant-bits first.
        for (index, item) in value.iter().rev().enumerate().take(8) {
            self.0 ^= u64::from(*item) << (index * 8);
        }
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

/// Alias for a `BuildHasherDefault<HashHasher>`.
pub type HashBuildHasher = BuildHasherDefault<HashHasher>;

/// Alias for a `std::collections::HashMap<K, V, HashBuildHasher>`.
pub type HashedMap<K, V> = ::std::collections::HashMap<K, V, HashBuildHasher>;

/// Alias for a `std::collections::HashSet<K, HashBuildHasher>`.
pub type HashedSet<K> = ::std::collections::HashSet<K, HashBuildHasher>;

#[cfg(test)]
mod tests {
    use super::{HashBuildHasher, HashHasher, HashedMap, HashedSet};
    use rand::{thread_rng, Rng};
    use std::hash::{Hash, Hasher};

    #[test]
    fn hasher() {
        let mut hash_hasher = HashHasher::default();
        hash_hasher.write(&[9]);
        assert_eq!(9, hash_hasher.finish());
        hash_hasher.write(&[4, 0]);
        assert_eq!(1033, hash_hasher.finish());
        hash_hasher.write(&[1, 4, 0]);
        assert_eq!(65545, hash_hasher.finish());

        hash_hasher = HashHasher::default();
        hash_hasher.write(&[3, 231]);
        assert_eq!(999, hash_hasher.finish());

        hash_hasher = HashHasher::default();
        hash_hasher.write(&[0, 0, 0, 0, 255, 255, 255, 255]);
        assert_eq!(4_294_967_295, hash_hasher.finish());

        hash_hasher = HashHasher::default();
        hash_hasher.write(&[255, 255, 255, 255, 255, 255, 255, 1]);
        assert_eq!(18_446_744_073_709_551_361, hash_hasher.finish());

        hash_hasher = HashHasher::default();
        hash_hasher.write(&[255, 255, 255, 255, 255, 255, 255, 255]);
        assert_eq!(18_446_744_073_709_551_615, hash_hasher.finish());

        hash_hasher = HashHasher::default();
        hash_hasher.write(&[0, 255, 255, 255, 255, 255, 255, 255, 255]);
        assert_eq!(18_446_744_073_709_551_615, hash_hasher.finish());

        hash_hasher = HashHasher::default();
        hash_hasher.write(&[
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 255, 255, 255, 255, 255, 255, 255, 255,
        ]);
        assert_eq!(18_446_744_073_709_551_615, hash_hasher.finish());
    }

    #[test]
    fn hash_map() {
        let mut map = HashedMap::with_capacity_and_hasher(3, HashBuildHasher::default());
        let mut sha1 = [0_u8; 20];
        assert!(map.insert(sha1, "First").is_none());
        sha1[19] = 1;
        assert!(map.insert(sha1, "Second").is_none());
        sha1[0] = 1;
        assert!(map.insert(sha1, "Third").is_none());
        assert_eq!(map.insert(sha1, "Fourth"), Some("Third"));
    }

    #[test]
    fn determinism() {
        let mut set1 = HashedSet::<u64>::default();
        let mut set2 = HashedSet::default();

        let mut set3 = ::std::collections::HashSet::new();
        let mut set4 = ::std::collections::HashSet::new();

        let mut rng = thread_rng();
        for _ in 0..100 {
            let random_value = rng.gen();
            let _ = set1.insert(random_value);
            let _ = set2.insert(random_value);
            let _ = set3.insert(random_value);
            let _ = set4.insert(random_value);
        }

        assert_eq!(
            set1.into_iter().collect::<Vec<_>>(),
            set2.into_iter().collect::<Vec<_>>()
        );
        assert_ne!(
            set3.into_iter().collect::<Vec<_>>(),
            set4.into_iter().collect::<Vec<_>>()
        );
    }

    fn hash<H: Hash>(value: H) -> u64 {
        let mut hasher = HashHasher::default();
        value.hash(&mut hasher);
        hasher.finish()
    }

    #[test]
    // This checks for regressions to https://github.com/Fraser999/Hash-Hasher/issues/1
    fn avoid_tending_towards_max_value() {
        let h1 = hash(&[u64::max_value()]);
        assert_ne!(u64::max_value(), h1);

        let h2 = hash(&[u64::max_value(), u64::max_value()]);
        assert_ne!(u64::max_value(), h2);
        assert_ne!(h1, h2, "\nh1: {:b}\nh2: {:b}\n", h1, h2);

        let h3 = hash(&[
            [u64::max_value(), u64::max_value()],
            [u64::max_value(), u64::max_value()],
        ]);
        assert_ne!(u64::max_value(), h3);
        assert_ne!(h1, h3, "\nh1: {:b}\nh3: {:b}\n", h1, h3);
        assert_ne!(h2, h3, "\nh2: {:b}\nh3: {:b}\n", h2, h3);
    }
}
