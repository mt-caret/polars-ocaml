# hash_hasher

A [`std::hash::Hasher`](https://doc.rust-lang.org/std/hash/trait.Hasher.html) which is designed to
work with already-hashed or hash-like data.

[![Documentation](https://docs.rs/hash_hasher/badge.svg)](https://docs.rs/hash_hasher)
[![](http://meritbadge.herokuapp.com/hash_hasher)](https://crates.io/crates/hash_hasher)
[![Build status](https://ci.appveyor.com/api/projects/status/cw65dk301auysvom/branch/master?svg=true)](https://ci.appveyor.com/project/Fraser999/hash-hasher/branch/master)
[![Build Status](https://travis-ci.org/Fraser999/Hash-Hasher.svg?branch=master)](https://travis-ci.org/Fraser999/Hash-Hasher)

## Details

The provided hasher does minimal work under the assumption that the input data is already suitable
for use as a key in a `HashSet` or `HashMap`.

As well as the performance benefit, it also causes `HashSet`s or `HashMap`s to become somewhat
deterministic.  Given two equal `HashSet`s or `HashMap`s containing more than a single element,
iterating them will yield the elements in differing orders.  By using a
[`hash_hasher::HashedSet`](https://docs.rs/hash_hasher/*/hash_hasher/type.HashedSet.html) or
[`hash_hasher::HashedMap`](https://docs.rs/hash_hasher/*/hash_hasher/type.HashedMap.html), then if
the same data is inserted and/or removed *in the same order*, iterating the collection will yield a
consistent order.

## Example

Since `new()` and `with_capacity()` aren't available for `HashSet`s or `HashMap`s using custom
hashers, the available constructors are `default()`, `with_hasher()` and
`with_capacity_and_hasher()`.

```rust
extern crate hash_hasher;

use hash_hasher::{HashBuildHasher, HashedMap, HashedSet};

let mut map = HashedMap::default();
assert!(map.insert(0, "zero").is_none());

let mut set = HashedSet::with_capacity_and_hasher(100, HashBuildHasher::default());
assert!(set.insert(0));
```

## Benchmarks

A benchmark suite is included and sample figures can be found at the end of the nightly jobs of the
[AppVeyor results](https://ci.appveyor.com/project/Fraser999/hash-hasher/branch/master) and the
[Travis results](https://travis-ci.org/Fraser999/Hash-Hasher).

For example:

```
insert_sha1s_into_set_using_default_hasher      ... bench:       1,171 ns/iter (+/- 30)
insert_sha1s_into_set_using_hash_hasher         ... bench:         533 ns/iter (+/- 9)

insert_sha256s_into_set_using_default_hasher    ... bench:       1,340 ns/iter (+/- 57)
insert_sha256s_into_set_using_hash_hasher       ... bench:         546 ns/iter (+/- 11)

insert_sha512s_into_set_using_default_hasher    ... bench:       1,804 ns/iter (+/- 2,597)
insert_sha512s_into_set_using_hash_hasher       ... bench:         704 ns/iter (+/- 22)

insert_sip_hashes_into_set_using_default_hasher ... bench:         781 ns/iter (+/- 33)
insert_sip_hashes_into_set_using_hash_hasher    ... bench:         256 ns/iter (+/- 50)
```

## License

Licensed under either of

* [Apache License, Version 2.0](https://opensource.org/licenses/Apache-2.0) (see also [LICENSE-APACHE](LICENSE-APACHE))
* [MIT License](https://opensource.org/licenses/MIT) (see also [LICENSE-MIT](LICENSE-MIT))

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
