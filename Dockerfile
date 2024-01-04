FROM --platform=amd64 ocaml/opam:debian-ocaml-4.14

LABEL org.opencontainers.image.source=https://github.com/mt-caret/polars-ocaml

RUN sudo ln -f /usr/bin/opam-2.2 /usr/bin/opam

RUN sudo apt-get update && sudo apt-get install -y \
    build-essential \
    curl

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain=nightly

ENV PATH="${HOME}/.cargo/bin:${PATH}"

RUN opam install dune ocamlformat ocaml-lsp-server \
    && cargo install cargo-watch

COPY ./polars.opam ./polars_async.opam ./

RUN opam install . --deps-only --with-doc --with-test
