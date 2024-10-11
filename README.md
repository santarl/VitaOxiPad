# VitaPad on Rust

<center>
    <img src="./server/sce_sys/icon0.png" width="128" height="128" />
    <p>Turn your PS Vita into a gamepad for your PC!</p>
    <sub>
        Based on<br>
        <a href="https://github.com/Rinnegatamante/VitaPad">VitaPad</a> by <a href="https://github.com/Rinnegatamante">Rinnegatamante</a>
        <br>and<br>
        <a href="https://github.com/saidsay-so/VitaPad">Rust-based VitaPad</a> by <a href="https://github.com/saidsay-so">saidsay-so</a>
    </sub>
</center>

## Description

VitaPad is a [client-server application](https://en.wikipedia.org/wiki/Client%E2%80%93server_model) that turns your PS Vita into (almost) a DualShock 4. The PS Vita acts as a server to which the PC client will be connected within the local network (WiFi).

The server part is written in C, the client part is written in Rust.

This will NOT work with [DS4Windows](https://github.com/Ryochan7/DS4Windows).

## What works

| Feature                            | Support | Details                                                                              |
| ---------------------------------- | :-----: | ------------------------------------------------------------------------------------ |
| Buttons                            |    ✅    | -                                                                                    |
| Sticks                             |    ✅    | -                                                                                    |
| L1 and R1                          |    ✅    | Press only, as the PS Vita does not have analog triggers. This emulates a full press |
| Select and Start                   |    ✅    | -                                                                                    |
| Use of the front PS Vita digitizer |    ✅    | Can be used to emulate L2\L3 and R2\R3                                               |
| Use of the back PS Vita digitizer  |    ✅    | Can be used to emulate L2\L3 and R2\R3                                               |
| Accelerometer                      |    ✅    | A little less accurate than the DS4, but still usable                                |
| Gyroscope                          |    ✅    | A little less accurate than the DS4, but still usable                                |
| DS4 digitizer emulation            |    ❌    | In progress...                                                                       |
| Sound                              |    ❌    | -                                                                                    |
| Any configuration                  |    ❌    | -                                                                                    |

## Installation

The server has to be installed on the PS Vita and the client on the PC.

### Requirements for Windows

- [ViGEmBus](https://github.com/ViGEm/ViGEmBus/releases)

## Client compilation 

### Windows

To compile on Windows you will need
- Make and Cmake;
- Rust (Cargo).

You can use [MSYS2](https://www.msys2.org/), which provides this in a convenient way.

Dependencies you will need for **MSYS2 MINGW64**:

```bash
pacman -S mingw-w64-x86_64-rust mingw-w64-x86_64-flatbuffers make git wget cmake
```

Clone the repository and log into it:

```bash
git clone --recurse-submodules https://github.com/DvaMishkiLapa/VitaPad.git
cd ./VitaPad
```

Build the client side:

```bash
cd ./client
cargo build --release --bin cli
```

You can check the finished binary by running in it:

```bash
 ./target/release/cli.exe your_PS_Vita_IP
```

### Server

1. Make sure that you have [`cmake`](https://cmake.org) installed.
2. Make sure you have [VitaSDK](https://vitasdk.org) installed and configured (try [vdpm](https://github.com/vitasdk/vdpm)).
3. Build the project with the following commands:

  ```bash
  cmake -S server -B build
  cmake --build build
  ```

Then, install the generated `VitaPad.vpk` file on your PS Vita.

## Thanks

Thanks to all the people who contributed to VitaSDK,
as well as Rinnegatamante for the original project and saidsay-so for the improved Rust version.
