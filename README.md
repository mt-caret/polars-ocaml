# polars-ocaml

🐻‍❄️❤️🐫

polars-ocaml is a project to provide idiomatic OCaml bindings to the Polars dataframe library.

## status

Currently very much WIP; see progress of examples ported from the Polars user guide
in the test directory.

Note that the current code assumes that the OCaml version is 4.14.1.

## license

This project is licensed under the terms of the MIT license, with the exception of the following files:
- `./test/data/iris.csv`: [the Iris dataset](https://archive.ics.uci.edu/dataset/53/iris) by R. A. Fisher is licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/legalcode).
- `./test/data/legislators-historical.csv`: [congress-legislators](https://github.com/unitedstates/congress-legislators) is licensed under [CC0](https://creativecommons.org/publicdomain/zero/1.0/legalcode)
- [`./test/data/pokemon.csv`](https://gist.github.com/ritchie46/cac6b337ea52281aa23c049250a4ff03/)
- [`./test/data/appleStock.csv`](https://github.com/pola-rs/polars-book/blob/4c7773952f73213326aa761599a779c9c2b3c94a/docs/src/data/appleStock.csv)