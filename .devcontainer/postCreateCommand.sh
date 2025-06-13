#!/usr/bin/env bash

set -euox pipefail
IFS=$'\n\t'

sudo chown vscode:vscode ./target/

just install-cargo-tools