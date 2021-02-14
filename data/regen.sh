#!/bin/bash

set -e

rm -fv data/input/us/seattle/raw_maps/huge_seattle.bin data/system/us/seattle/maps/huge_seattle.bin data/input/us/seattle/raw_maps/huge_seattle.bin data/input/us/seattle/popdat.bin

./import.sh --regen_all

cargo run --release --bin game -- --prebake

cargo run --release --bin tests
