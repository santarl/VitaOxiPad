# VitaPad v1.3

<center>
<img src="./server/sce_sys/icon0.png" width="128" height="128" />
<p>Turn your PS Vita into a gamepad for your PC!</p>
<sub>Based on <a href="https://github.com/Rinnegatamante/VitaPad">VitaPad</a> by <a href="https://github.com/Rinnegatamante">Rinnegatamante</a></sub>
</center>

## Installation

The server has to be installed on the PS Vita and the client on the PC.

### Requirements

- [VitaSDK](https://vitasdk.org/)
- [CMake](https://cmake.org/)
- [flatc](https://google.github.io/flatbuffers/flatbuffers_guide_using_schema_compiler.html)

#### Windows

- [ViGEmBus](https://github.com/ViGEm/ViGEmBus/releases)

### Server

```bash
cmake -S server -B build
cmake --build build
```

Then, install the generated `VitaPad.vpk` file on your PS Vita.

### Client

```bash
cd client
cargo build --release --bin cli
```

Then, run the generated executable (target/release/cli{.exe}) on your PC.

## Thanks

Thanks to all the people who contributed to VitaSDK, and Rinnegatamante for the original project.
