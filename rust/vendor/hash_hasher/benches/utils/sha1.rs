use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

const DIGEST_SIZE: usize = 20;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Digest(pub [u8; DIGEST_SIZE]);

impl Distribution<Digest> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Digest {
        Digest(rng.gen())
    }
}
