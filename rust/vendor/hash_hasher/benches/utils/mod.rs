pub mod sha1;
pub mod sha256;
pub mod sha512;
pub mod sip;

use fnv::{FnvBuildHasher, FnvHasher};
use hash_hasher::{HashBuildHasher, HashHasher};
use rand::{
    self,
    distributions::{Distribution, Standard},
};
use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    hash::{BuildHasher, Hash, Hasher},
};
use test::Bencher;

const COUNT: usize = 32;

fn random_data<T>() -> Vec<T>
where
    Standard: Distribution<T>,
{
    let mut data = Vec::<T>::with_capacity(COUNT);
    for _ in 0..COUNT {
        data.push(rand::random());
    }
    data
}

fn perform_hash<T, H>(bencher: &mut Bencher)
where
    T: Hash,
    H: Hasher + Default,
    Standard: Distribution<T>,
{
    let data = random_data::<T>();
    bencher.iter(|| {
        for element in &data {
            let mut hasher = H::default();
            element.hash(&mut hasher);
            let _ = hasher.finish();
        }
    });
}

fn insert_to_set<T, S>(set: &mut HashSet<T, S>, bencher: &mut Bencher)
where
    T: Copy + Eq + Hash,
    S: BuildHasher,
    Standard: Distribution<T>,
{
    let data = random_data();
    bencher.iter(|| {
        for element in &data {
            let _ = set.insert(*element);
        }
    });
    assert!(!set.is_empty());
}

pub fn hash_using_default_hasher<T>(bencher: &mut Bencher)
where
    T: Hash,
    Standard: Distribution<T>,
{
    perform_hash::<T, DefaultHasher>(bencher);
}

pub fn hash_using_hash_hasher<T>(bencher: &mut Bencher)
where
    T: Hash,
    Standard: Distribution<T>,
{
    perform_hash::<T, HashHasher>(bencher);
}

pub fn hash_using_fnv_hasher<T>(bencher: &mut Bencher)
where
    T: Hash,
    Standard: Distribution<T>,
{
    perform_hash::<T, FnvHasher>(bencher);
}

pub fn set_using_default_hasher<T>(bencher: &mut Bencher)
where
    T: Copy + Eq + Hash,
    Standard: Distribution<T>,
{
    let mut set = HashSet::<T>::with_capacity(COUNT);
    insert_to_set(&mut set, bencher);
}

pub fn set_using_hash_hasher<T>(bencher: &mut Bencher)
where
    T: Copy + Eq + Hash,
    Standard: Distribution<T>,
{
    let hash_builder = HashBuildHasher::default();
    let mut set = HashSet::<T, HashBuildHasher>::with_capacity_and_hasher(COUNT, hash_builder);
    insert_to_set(&mut set, bencher);
}

pub fn set_using_fnv_hasher<T>(bencher: &mut Bencher)
where
    T: Copy + Eq + Hash,
    Standard: Distribution<T>,
{
    let hash_builder = FnvBuildHasher::default();
    let mut set = HashSet::<T, FnvBuildHasher>::with_capacity_and_hasher(COUNT, hash_builder);
    insert_to_set(&mut set, bencher);
}
