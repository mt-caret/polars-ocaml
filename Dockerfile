FROM ocaml/opam:debian-ocaml-4.14

LABEL org.opencontainers.image.source=https://github.com/mt-caret/polars-ocaml

RUN sudo ln -f /usr/bin/opam-2.2 /usr/bin/opam

RUN sudo apt-get update && sudo apt-get install -y \
    build-essential \
    curl \
    mold

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain=nightly

ENV PATH="${HOME}/.cargo/bin:${PATH}"

RUN opam install dune ocamlformat ocaml-lsp-server \
    && cargo install cargo-watch

COPY --chown=opam ./polars.opam ./polars_async.opam ./

RUN opam install . --deps-only --with-doc --with-test

# Overwrite default linker with mold (this drastically speeds up builds)
RUN sudo ln -f /usr/bin/mold "$(realpath /usr/bin/ld)"

COPY --chown=opam . .
