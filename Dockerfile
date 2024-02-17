FROM ocaml/opam:ubuntu-22.04-ocaml-4.14

LABEL org.opencontainers.image.source=https://github.com/mt-caret/polars-ocaml

RUN DEBIAN_FRONTEND=noninteractive sudo apt-get update && sudo apt-get install -y \
    curl \
    build-essential \
    mold

# Overwrite default linker with mold (this drastically speeds up builds)
RUN sudo ln -sf /usr/bin/mold "$(realpath /usr/bin/ld)"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain=nightly
ENV PATH="${HOME}/.cargo/bin:${PATH}"

RUN sudo ln -f /usr/bin/opam-2.2 /usr/bin/opam
RUN opam install dune ocamlformat ocaml-lsp-server --yes && cargo install cargo-watch

COPY --chown=opam ./polars.opam ./polars_async.opam ./
RUN opam install . --deps-only --with-doc --with-test --assume-depexts --yes
