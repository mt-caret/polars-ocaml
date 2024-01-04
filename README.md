# polars-ocaml

🐻‍❄️❤️🐫

polars-ocaml is a project to provide idiomatic OCaml bindings to the Polars
dataframe library.

Check out the [ported examples from the Polars user guide](./guide/) for a quick tour of what you
can do with polars-ocaml!

Install with `opam install polars`.

## project status

![build](https://github.com/mt-caret/polars-ocaml/actions/workflows/build.yaml/badge.svg)

Currently very much WIP. Please expect breakages and large changes in API.

Note that the current code assumes that the OCaml version is 4.14.1;
see [HACKING.md](./HACKING.md) for notes on how to build polars-ocaml.

## docs

odoc-generated docs are available [here](https://mt-caret.github.io/polars-ocaml/polars/Polars/index.html).

## utop

polars-ocaml works in utop!

`$ dune utop --profile utop`[^utop-workaround]

[^utop-workaround]: The special profile is a workaround for a [known limitation in a library we use](https://github.com/tizoc/ocaml-interop/issues/49#issuecomment-1627842973)).

![polars-ocaml running in utop](https://user-images.githubusercontent.com/4996739/253110945-c8ffb606-bcbb-4297-acef-602d3cecd15b.png)

## jupyter notebook

polars-ocaml works in Jupyter notebooks, using [akabe/ocaml-jupyter](https://github.com/akabe/ocaml-jupyter)!

Just install the `polars` opam package alongside the `jupyter` package:

```
$ pip install jupyter
$ opam install jupyter polars
$ ocaml-jupyter-opam-genspec
$ jupyter kernelspec install --user --name "ocaml-jupyter-$(opam var switch)" "$(opam var share)/jupyter"
```

![polars-ocaml running in jupyter](https://github.com/mt-caret/polars-ocaml/assets/4996739/fa09ef51-078f-4aad-af39-1243ca559451)

## license

This project is licensed under the terms of the MIT license, with the exception
of the following files:
- `./guide/data/iris.csv`: [the Iris dataset](https://archive.ics.uci.edu/dataset/53/iris) by R. A. Fisher is licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/legalcode).
- `./guide/data/legislators-historical.csv`: [congress-legislators](https://github.com/unitedstates/congress-legislators) is licensed under [CC0](https://creativecommons.org/publicdomain/zero/1.0/legalcode)
- [`./guide/data/pokemon.csv`](https://gist.github.com/ritchie46/cac6b337ea52281aa23c049250a4ff03/)
- [`./guide/data/appleStock.csv`](https://github.com/pola-rs/polars-book/blob/4c7773952f73213326aa761599a779c9c2b3c94a/docs/src/data/appleStock.csv)
- [`./guide/data/reddit.csv`](https://github.com/pola-rs/polars-book/blob/4c7773952f73213326aa761599a779c9c2b3c94a/docs/src/data/reddit.csv)
