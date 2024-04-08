@echo off

cargo build --release --workspace

cd target/release
if not exist "plugins/" mkdir "plugins/"
del padamo_api_macros_internal.dll
del padamo_iced_forms.dll
move *.dll plugins
