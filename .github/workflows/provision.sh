#!/bin/bash
# This script contains the commands for provisioning the LXD container based on
# the default ubuntu 22.04 image, to be used in tandem with the scripts in
# https://github.com/stgraber/lxd-github-actions to set up self-hosted github
# action runners on LXD.
set -euxo pipefail

sudo apt update
# install opam + mold
sudo apt install --yes \
    curl \
    opam \
    mold
opam init --auto-setup --compiler=4.14.1

# install rustup, and make nightly toolchain default
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
sh -s -- -y --default-toolchain=nightly