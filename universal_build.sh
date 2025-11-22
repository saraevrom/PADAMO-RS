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

BUILD_TARGET=$1
shift 1

case "${BUILD_TARGET}" in
 debug)
    cargo build --workspace $@ || exit 1
    cargo build || exit 1
    ;;
 release)
    cargo build --release --workspace $@ || exit 1
    cargo build --release $@ || exit 1
    ;;
 *)
 echo "Unsupported target"

esac


cp  -rv "./assets" "./target/${BUILD_TARGET}/"

cd "./target/${BUILD_TARGET}/"


#Preserving config
[ -f "plugins/padamo-neuraltrigger/config.toml" ] && mv -v plugins/padamo-neuraltrigger/config.toml ann_config.toml

mkdir -pv plugins
rm -rvf plugins/*



# ANN trigger subdir
rm -rf padamo-neuraltrigger
mkdir -pv padamo-neuraltrigger
mv libpadamoneuraltrigger.so padamo-neuraltrigger/
cp -fv ../../padamo-neuraltrigger/*.onnx padamo-neuraltrigger/
[ -f "ann_config.toml" ] && mv -v ann_config.toml padamo-neuraltrigger/config.toml

mv padamo-neuraltrigger/ plugins/


# Base modules
mv -v libpadamobasesignalprocessing.so  plugins/
mv -v libpadamoflatfielding.so          plugins/
mv -v libpadamomat.so                   plugins/
mv -v libpadamobasictriggers.so         plugins/
mv -v libpadamofunctions.so             plugins/
mv -v libpadamosignalmanipulation.so    plugins/
mv -v libpadamocore.so                  plugins/
mv -v libpadamohdf5.so                  plugins/
mv -v libpadamotrackgen.so              plugins/
mv -v libpadamorandom.so                plugins/
mv -v libpadamostft.so                  plugins/
mv -v libpadamoeusoroot.so              plugins/
mv -v libpadamoplaintext.so             plugins/
mv -v libpadamotransforms.so            plugins/
