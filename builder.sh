#!/bin/bash
#
# build_crate(){
#     echo "BUILDING CRATE $1"
#     (cd "$1"; cargo build) || exit 1
#     cp -v "./$1/target/debug"/*.so ./target/debug/plugins/
# }
#
#
# cargo build || exit 1
# (cd ./target/debug/ ; mkdir -p plugins)
# build_crate padamo-core
# build_crate padamo-hdf5
# build_crate padamo-base-processing
# build_crate padamo-signal-manipulation
# build_crate padamo-basic-triggers


cargo build --workspace || exit 1
cd ./target/debug/
rm libpadamo_api_macros_internal.so  #  Padamo API internal library: not a plugin
rm libpadamo_iced_forms.so           #  Padamo iced forms. Also not a plugin
mkdir -pv plugins
rm -r plugins/*
mv -v *.so plugins/
