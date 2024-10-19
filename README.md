# VitaOxiPad <img align="right" width="128" height="128" src="./server/sce_sys/icon0.png" />

Turn your PS Vita into a gamepad for your PC!</p>

Based on [VitaPad by Rinnegatamante](https://github.com/Rinnegatamante/VitaPad) and [Rust-based VitaPad by saidsay-so](https://github.com/saidsay-so/VitaPad).

- [VitaOxiPad ](#vitaoxipad-)
  - [Description](#description)
  - [What works](#what-works)
  - [Uses and options](#uses-and-options)
    - [Configurations](#configurations)
  - [Bugs](#bugs)
  - [Installation](#installation)
    - [Requirements for Windows](#requirements-for-windows)
  - [Client compilation](#client-compilation)
    - [Windows](#windows)
    - [Server](#server)
  - [Thanks](#thanks)
  - [Why Oxi?](#why-oxi)

## Description

VitaOxiPad is a [client-server application](https://en.wikipedia.org/wiki/Client%E2%80%93server_model) that turns your PS Vita into (almost) a DualShock 4. The PS Vita acts as a server to which the PC client will be connected within the local network (WiFi).

The server part is written in C, the client part is written in Rust.

This will NOT work with [DS4Windows](https://github.com/Ryochan7/DS4Windows).

## What works

| Feature                         | Support | Details                                                                              |
| ------------------------------- | :-----: | ------------------------------------------------------------------------------------ |
| Buttons                         |    ✅    | -                                                                                    |
| Sticks                          |    ✅    | -                                                                                    |
| L1 and R1                       |    ✅    | Press only, as the PS Vita does not have analog triggers. This emulates a full press |
| L2/R2 and L3/R3 emulation       |    ✅    | Can be used of the back or front PS Vita digitizer for it                            |
| Select and Start                |    ✅    | -                                                                                    |
| Accelerometer and gyroscope     |    ✅    | A little less accurate than the DualShock 4, but still usable                        |
| DualShock 4 digitizer emulation |    ✅    | Emulates up to two-finger simultaneous input, same as DualShock 4                    |
| Any configuration               |    ✅    | You can choose from [ready-made configurations](#configurations)                     |
| Sound                           |    ❌    | -                                                                                    |

## Uses and options

```bash
$ VitaOxiPad-x64.exe --help
Usage: VitaOxiPad-x64.exe <ip> [-p <port>] [-c <config>] [--polling-interval <polling-interval>] [-d]

Create a virtual controller and fetch its data from a Vita over the network.

Positional Arguments:
  ip                IP address of the Vita to connect to

Options:
  -p, --port        port to connect to (default: 5000)
  -c, --config      buttons and touchpads config (default: standart)
  --polling-interval
                    polling interval in microseconds (minimum = 4000)
  -d, --debug       enable debug mode
  -v, --version     show version information
  --help            display usage information
```

### Configurations

There are currently 4 DualShock 4 configurations emulations that can be selected at client startup:

| Configurations name | PS Vita L1\R1 |        PS Vita front digitizer         |         PS Vita rear digitizer         |
| ------------------- | :-----------: | :------------------------------------: | :------------------------------------: |
| `standart`          |     L1\R1     |                 L3\R3                  |                 L2\R2                  |
| `alt_triggers`      |     L2\R2     |                 L3\R3                  |                 L1\R1                  |
| `rear_touchpad`     |     L1\R1     | upper area - L2\R2, lower area - L3\R3 |         DualShock 4 digitizer          |
| `front_touchpad`    |     L1\R1     |         DualShock 4 digitizer          | upper area - L2\R2, lower area - L3\R3 |

To better understand the emulation behavior, you can run [3D Controller Overlay](http://www.3d-controller-overlay.org/) after connecting your PS Vita.

## Bugs

- Rarely, a server on PS Vita can crash with an error. The causes are being investigated. Happened to me more than once in 3-4 hours of play;
- Sometimes, the imput-lag increases a lot. This may be due to a bad WiFi connection.

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
git clone --recurse-submodules https://github.com/DvaMishkiLapa/VitaOxiPad.git
cd ./VitaOxiPad
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

1. Make sure that you have [`cmake`](https://cmake.org) installed;
2. Make sure you have [VitaSDK](https://vitasdk.org) installed and configured (try [vdpm](https://github.com/vitasdk/vdpm));

  Sometimes, for whatever reason, `flatbuffers` are not installed in the VitaSDK (`fatal error: flatbuffers/flatbuffers.h: No such file or directory`).

  You can install it manually via vdpm. Do this after installing the VitaSDK via vdpm:

  ```bash
  ./vdpm flatbuffers
  ```

3. Install [`flatc`](https://flatbuffers.dev/flatbuffers_guide_building.html) for your system. For Linux:

  ```bash
  git clone https://github.com/google/flatbuffers.git && cd flatbuffers
  cmake -G "Unix Makefiles" -DCMAKE_BUILD_TYPE=Release
  make -j
  ./flattests # this is quick, and should print "ALL TESTS PASSED"
  sudo make install
  sudo ldconfig
  flatc --version
  ```

4. Build the project with the following commands:

  ```bash
  cmake -S server -B build
  cmake --build build
  ```

Then, install the generated `VitaOxiPad.vpk` file on your PS Vita.

## Thanks

- Thanks to all the people who contributed to [VitaSDK](https://vitasdk.org), [Vitadev Package manager](https://github.com/vitasdk/vdpm) and [ViGEm Bus Driver](https://github.com/nefarius/ViGEmBus);
- [Rinnegatamante](https://github.com/Rinnegatamante) for [the original VitaPad version](https://github.com/Rinnegatamante/VitaPad);
- [saidsay-so](https://github.com/saidsay-so) for [the improved Rust VitaPad version](https://github.com/saidsay-so/VitaPad);
- [santarl](https://github.com/santarl) for advice and help with accelerometer, gyroscope and also adding DualShock 4 digitizer emulation;
- [CasualX](https://github.com/CasualX) for [ViGEm client in Rust](https://github.com/CasualX/vigem-client).

## Why Oxi?

Now there are many projects with the name VitaPad. It's very confusing.

Oxi - oxidation, the process that causes **rust**ing.
