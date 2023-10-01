FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    curl \
    opam \
    mold
RUN opam init --auto-setup --compiler=4.14.1 --disable-sandboxing

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain=nightly

RUN opam install dune ocamlformat --yes

COPY ./polars.opam ./polars_async.opam .
RUN opam install . --deps-only --with-doc --with-test --yes
