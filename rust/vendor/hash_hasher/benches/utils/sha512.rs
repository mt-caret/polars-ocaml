use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

const DIGEST_SIZE: usize = 64;

#[derive(Copy, Clone)]
pub struct Digest(pub [u8; DIGEST_SIZE]);

impl PartialEq for Digest {
    fn eq(&self, other: &Self) -> bool {
        self.0[..] == other.0[..]
    }
}

impl Eq for Digest {}

impl PartialOrd for Digest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Digest {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0[..].cmp(&other.0[..])
    }
}

impl Hash for Digest {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.0[..])
    }
}

impl Distribution<Digest> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Digest {
        let mut digest = [0_u8; DIGEST_SIZE];
        for c in digest[..].iter_mut() {
            *c = rng.gen();
        }
        Digest(digest)
    }
}
