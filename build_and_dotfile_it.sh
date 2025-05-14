#!/bin/bash

# This script builds skoll and then copies the exec to ~/.config/niri

cargo build --release;
cp ~/Repositories/skoll/target/release/skoll ~/.config/niri/;
echo -e "\n\033[0;32m\033[1mskoll built and copied to ~/.config/niri\033[0m";