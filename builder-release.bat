@echo off

cargo build --release --workspace

cd target/release

if not exist "padamo-neuraltrigger/" mkdir "padamo-neuraltrigger/"
del padamo-neuraltrigger/*
move padamoneuraltrigger.dll padamo-neuraltrigger/
copy -fv ../../padamo-neuraltrigger/*.onnx padamo-neuraltrigger/
move onnx*.dll padamo-neuraltrigger/

if not exist "plugins/" mkdir "plugins/"
del padamo_api_macros_internal.dll
del padamo_iced_forms.dll
move *.dll plugins

move  padamo-neuraltrigger/ plugins/padamo-neuraltrigger
