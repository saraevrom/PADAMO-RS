@echo off

cargo build --release --workspace

xcopy /y /e assets\* target\release\assets\*

cd target/release


echo Preparing ANN triggers
if exist "padamo-neuraltrigger/" rmdir /Q /S "padamo-neuraltrigger/"
mkdir "padamo-neuraltrigger/"
move padamoneuraltrigger.dll "padamo-neuraltrigger/"

echo Shipping ANNs
copy ..\..\padamo-neuraltrigger\*.onnx padamo-neuraltrigger

echo Preparing plugins
if exist "plugins/" rmdir /Q /S "plugins/"
mkdir "plugins/"

del padamo_api_macros_internal.dll
del padamo_iced_forms.dll


rem Base modules
move padamobasesignalprocessing.dll  plugins
move padamoflatfielding.dll          plugins
move padamomat.dll                   plugins
move padamobasictriggers.dll         plugins
move padamofunctions.dll             plugins
move padamosignalmanipulation.dll    plugins
move padamocore.dll                  plugins
move padamohdf5.dll                  plugins
move padamotrackgen.dll              plugins
move padamorandom.dll                plugins
move padamostft.dll                  plugins
move padamoeusoroot.dll              plugins
move padamoplaintext.dll             plugins

move /Y padamo-neuraltrigger plugins\padamo-neuraltrigger

cd ../../
