FROM ubuntu:22.04

LABEL org.opencontainers.image.source=https://github.com/mt-caret/polars-ocaml

RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    opam \
    mold
RUN opam init --auto-setup --compiler=4.14.1 --disable-sandboxing

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain=nightly
ENV PATH="/root/.cargo/bin:${PATH}"

RUN opam install dune ocamlformat ocaml-lsp-server --yes
RUN cargo install cargo-watch

COPY ./polars.opam ./polars_async.opam ./
RUN opam install . --deps-only --with-doc --with-test --assume-depexts --yes

# Overwrite default linker with mold (this drastically speeds up builds)
RUN ln -sf /usr/bin/mold "$(realpath /usr/bin/ld)"
