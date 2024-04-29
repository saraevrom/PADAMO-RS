# PADAMO-RS
Rust edition of PADAMO: the universal detector processing software.

# Building instructions
## Linux
You will need (along with git):
* Rust compiler (comes with cargo).
* cmake
* other usual building stuff (like gcc, make, etc) to build some crates like hdf5 and zlib

Building instructions (after cloning repo):

```bash
./builder-release.sh
cargo build --release
cargo run --release
```

## Windows
You will need (along with git):
* Rust compiler (comes with cargo). Default installation option will be enough.
* cmake

Building instructions (from cmd, after cloning repo):

```cmd
builder-release.bat
cargo build --release
cargo run --release
```
