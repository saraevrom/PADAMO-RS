#!/bin/bash

# build_crate(){
#     echo "BUILDING CRATE $1"
#     (cd "$1"; cargo build --release) || exit 1
#     #pwd
#     #ls "./$1/target/debug/libpadamocore.so"
#     cp -v "./$1/target/release"/*.so ./target/release/plugins/
# }
#
# cargo build --release || exit 1
# (cd ./target/release/ ; mkdir -p plugins)
# build_crate padamo-core
# build_crate padamo-hdf5
# build_crate padamo-base-processing
# build_crate padamo-signal-manipulation
# build_crate padamo-basic-triggers
#


cargo build --release --workspace || exit 1
cd ./target/release/
rm libpadamo_api_macros_internal.so  #  Padamo API internal library: not a plugin
rm libpadamo_iced_forms.so           #  Padamo iced forms. Also not a plugin
mkdir -pv plugins
rm -r plugins/*
mv -v *.so plugins/
