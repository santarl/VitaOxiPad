# VitaOxiPad <img align="right" width="128" height="128" src="./server/sce_sys/icon0.png" />

Turns your PS Vita into (almost) a DualShock 4!

Based on [VitaPad by Rinnegatamante](https://github.com/Rinnegatamante/VitaPad) and [Rust-based VitaPad by saidsay-so](https://github.com/saidsay-so/VitaPad).

- [VitaOxiPad ](#vitaoxipad-)
  - [1. Description](#1-description)
  - [2. Installation](#2-installation)
  - [3. Uses and options](#3-uses-and-options)
    - [3.1 Saving Configs](#31-saving-configs)
    - [3.2 Using a Static IP](#32-using-a-static-ip)
    - [3.3 Configurations](#33-configurations)
  - [4. What works](#4-what-works)
    - [4.1 Windows Client](#41-windows-client)
    - [4.2 Linux Client](#42-linux-client)
  - [5. Client compilation](#5-client-compilation)
    - [5.1 Dependencies](#51-dependencies)
      - [5.1.1 Windows](#511-windows)
      - [5.1.2 Linux](#512-linux)
    - [5.2 Clone and Compile](#52-clone-and-compile)
  - [6. PS Vita server compilation](#6-ps-vita-server-compilation)
  - [FAQ](#faq)
    - [Q1: Why Oxi?](#q1-why-oxi)
    - [Q2: Why isn't my PS Vita connecting to the PC?](#q2-why-isnt-my-ps-vita-connecting-to-the-pc)
    - [Q3: I'm experiencing input lag. What can I do?](#q3-im-experiencing-input-lag-what-can-i-do)
    - [Q4: How do I update VitaOxiPad?](#q4-how-do-i-update-vitaoxipad)
  - [Thanks](#thanks)

<div align="center">
  <img src="./demo/demo.gif" width="60%" height="auto" alt="demo" />
</div>

## 1. Description

**VitaOxiPad** is a [client-server application](https://en.wikipedia.org/wiki/Client%E2%80%93server_model) that turns your PS Vita into (almost) a DualShock 4.
The PS Vita acts as a server to which the PC client will be connected within the local network (WiFi).

## 2. Installation

To set up VitaOxiPad, follow these steps:

1. **Install the Server on PS Vita:**
   - Ensure your PS Vita is set up for homebrew applications.
   - Download the `VitaOxiPad.vpk` file.
   - Use VitaShell or another package manager to install the `.vpk` on your PS Vita.

2. **Install the Client on Windows PC:**
   - Download the latest `VitaOxiPad-x64.exe` or `VitaOxiPad-x32.exe` from the [releases page](https://github.com/DvaMishkiLapa/VitaOxiPad/releases).
   - Install [ViGEmBus](https://github.com/ViGEm/ViGEmBus/releases) on your PC.
   - Run the `VitaOxiPad-x64.exe` or `VitaOxiPad-x32.exe` and enter your PS Vita's IP address.

## 3. Uses and options

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
                    polling interval in microseconds (minimum = 6000)
  -d, --debug       enable debug mode
  -v, --version     show version information
  -s, --sample-config
                    print sample config file
  --help            display usage information
```

### 3.1 Saving Configs

VitaOxiPad searches for the config file in the following locations:

- **Current Executable Directory**: `config.toml`
- **User's Home Directory**: `~/vitaoxipad.toml` or `~/.config/vitaoxipad.toml`
- **Windows Path**: `C:\Users\%username%\vitaoxipad\vitaoxipad.toml`

If no config file is found, default settings will be used.

With the `--sample-config` feature, you can generate a sample configuration file for VitaOxiPad.
This output can be redirected to a `vitaoxipad.toml` file, allowing you to save your configuration options and avoid the need to specify flags each time you run the application.

To create a config file, run the following command:

```bash
VitaOxiPad-x64.exe --sample-config > vitaoxipad.toml
```

This will create a `vitaoxipad.toml` file in the current directory with sample configuration options.

### 3.2 Using a Static IP

If your router allows it, A static IP can be assigned to the PS Vita in the Wi-Fi router settings, allowing this IP address to be saved in the vitaoxipad.toml configuration file.
This setup enables VitaOxiPad to run without the need to specify the IP address as a flag each time.

To set this up, the vitaoxipad.toml file should be edited to include the static IP address in the ip_address field:

```toml
ip = "PSVITA_STATIC_IP_ADDRESS"
```

Now, you can launch VitaOxiPad without any flags, and it will automatically use the IP address specified in the configuration file.

### 3.3 Configurations

There are currently 4 DualShock 4 configurations emulations that can be selected at client startup:

| Configurations name | PS Vita L1\R1 |        PS Vita front digitizer         |         PS Vita rear digitizer         |
| ------------------- | :-----------: | :------------------------------------: | :------------------------------------: |
| `standart`          |     L2\R2     |                 L3\R3                  |                 L1\R1                  |
| `alt_triggers`      |     L1\R1     |                 L3\R3                  |                 L2\R2                  |
| `rear_touchpad`     |     L2\R2     | upper area - L1\R1, lower area - L3\R3 |         DualShock 4 digitizer          |
| `front_touchpad`    |     L2\R2     |         DualShock 4 digitizer          | upper area - L1\R1, lower area - L3\R3 |

To better understand the emulation behavior,
you can run [3D Controller Overlay](http://www.3d-controller-overlay.org/) after connecting your PS Vita for Windows
or [evtest-qt](https://github.com/Grumbel/evtest-qt) for Linux.

## 4. What works

### 4.1 Windows Client

The gamepad chosen for emulation was a DualShock 4 v1 (`vendor: 0x054C`, `product: 0x05C4`) connected via USB.
This allows to support a large number of Windows games without any fixes.

| Feature                         | Support | Details                                                                                   |
| ------------------------------- | :-----: | ----------------------------------------------------------------------------------------- |
| Dpad, Sticks and Buttons        |    ✅    | -                                                                                         |
| L1 and R1                       |    ✅    | It can be used for L\R presses. For L2\R2, a full press is emulated                       |
| L2/R2 and L3/R3 emulation       |    ✅    | Can be emulated via the PS Vita's rear or front digitizer, as well as the PS Vita's L1\R1 |
| Accelerometer and gyroscope     |    ✅    | A little less accurate than the DualShock 4, but still usable                             |
| DualShock 4 digitizer emulation |    ✅    | Emulates up to two-finger simultaneous input, same as DualShock 4                         |
| DualShock 4 digitizer button    |    ✅    | Works as a quick tap on the digitizer. Supports front and rear digitizer                  |
| Battery                         |    ✅    | PS Vita's battery status is sent to the emulated DualShock 4                              |
| Any configuration               |    ✅    | You can choose from [ready-made configurations](#configurations)                          |
| DS4Windows support              |    ✅    | Virtual Controller Support[*] requires activation                                         |
| Sound                           |    ❌    | Probably will never be realized                                                           |

[*] - Virtual Controller Support can be found in the [schmaldeo DS4Windows fork](https://github.com/schmaldeo/DS4Windows).
This option can be found in `Settings -> Device Options -> Virtual Controller Support`.

### 4.2 Linux Client

The behavior is close to how the DualShock 4 (`vendor: 0x054C`, `product: 0x9CC`) is defined in Linux. Three `uinput` devices are emulated:

- Main `uinput` device (Dpad, Sticks, buttons, triggers)
- Touchpad `uinput` device;
- Motion Sensors `uinput` device for accelerometer and gyroscope.

| Feature                         | Support | Details                                                                                                         |
| ------------------------------- | :-----: | --------------------------------------------------------------------------------------------------------------- |
| Dpad, Sticks and Buttons        |    ✅    | -                                                                                                               |
| L1 and R1                       |    ✅    | It can be used for L\R presses. For L2\R2, a full press is emulated                                             |
| L2/R2 and L3/R3 emulation       |    ✅    | Can be emulated via the PS Vita's rear or front digitizer, as well as the PS Vita's L1\R1                       |
| Accelerometer and gyroscope     |    ✅    | A little less accurate than the DualShock 4, but still usable                                                   |
| DualShock 4 digitizer emulation |    ✅    | Emulates up to six-finger simultaneous input. Linux defines it like a touchpad                                  |
| DualShock 4 digitizer button    |    ✅    | Works as a quick tap on the digitizer. Supports front and rear digitizer. Linux itself determines the behaviour |
| Battery                         |    ❌    | -                                                                                                               |
| Any configuration               |    ✅    | You can choose from [ready-made configurations](#configurations)                                                |
| Sound                           |    ❌    | Probably will never be realized                                                                                 |

## 5. Client compilation

### 5.1 Dependencies

To compile the Client you will need

- [Make](https://www.gnu.org/software/make) and [Cmake](https://cmake.org);
- [Flatbuffers (flatc)](https://github.com/google/flatbuffers);
- [Rust](https://www.rust-lang.org/learn) ([cargo](https://doc.rust-lang.org/cargo)).

#### 5.1.1 Windows

You can use [MSYS2](https://www.msys2.org), which provides this in a convenient way.

Dependencies you will need for **MSYS2 MINGW64**:

```bash
pacman -S mingw-w64-x86_64-rust mingw-w64-x86_64-flatbuffers make git wget cmake
```

#### 5.1.2 Linux

Install the necessary dependencies in a way that is convenient for you.

### 5.2 Clone and Compile

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

## 6. PS Vita server compilation

1. Make sure that you have [`cmake`](https://cmake.org) installed;
1. Make sure you have [VitaSDK](https://vitasdk.org) installed and configured (try [vdpm](https://github.com/vitasdk/vdpm));

  Sometimes, for whatever reason, `flatbuffers` are not installed in the VitaSDK (`fatal error: flatbuffers/flatbuffers.h: No such file or directory`).

  You can install it manually via vdpm. Do this after installing the VitaSDK via vdpm:

  ```bash
  ./vdpm flatbuffers
  ```

1. Install [`flatc` v24.3.25](https://flatbuffers.dev/flatbuffers_guide_building.html) for your system. For Linux:

  ```bash
  git clone --branch v24.3.25 https://github.com/google/flatbuffers.git && cd flatbuffers
  cmake -G "Unix Makefiles" -DCMAKE_BUILD_TYPE=Release
  make -j
  ./flattests # this is quick, and should print "ALL TESTS PASSED"
  sudo make install
  sudo ldconfig
  flatc --version # "flatc version 24.3.25"
  ```

1. Build the project with the following commands:

  ```bash
  cmake -S server -B build
  cmake --build build
  ```

Then, install the generated `VitaOxiPad.vpk` file on your PS Vita.

## FAQ

### Q1: Why Oxi?

**A:** Now there are many projects with the name VitaPad. It's very confusing.
In this implementation, the client application was written using **Rust**.
**Oxi** - **oxi**dation, the process that causes **rust**ing.

### Q2: Why isn't my PS Vita connecting to the PC?

**A:** Ensure both devices are on the same WiFi network and that the IP address entered in the client is correct.

### Q3: I'm experiencing input lag. What can I do?

**A:** Input lag might be due to a poor WiFi connection. Try moving closer to the router or reducing network congestion.

### Q4: How do I update VitaOxiPad?

**A:** Check the [releases page](https://github.com/DvaMishkiLapa/VitaOxiPad/releases) for the latest version and follow the installation instructions provided.
Or you can use [VitaDB-Downloader](https://github.com/Rinnegatamante/VitaDB-Downloader) on your PS Vita.

## Thanks

- Thanks to all the people who contributed to [VitaSDK](https://vitasdk.org), [Vitadev Package manager](https://github.com/vitasdk/vdpm) and [ViGEm Bus Driver](https://github.com/nefarius/ViGEmBus);
- [Rinnegatamante](https://github.com/Rinnegatamante) for [the original VitaPad version](https://github.com/Rinnegatamante/VitaPad);
- [saidsay-so](https://github.com/saidsay-so) for [the improved Rust VitaPad version](https://github.com/saidsay-so/VitaPad);
- [santarl](https://github.com/santarl) for advice and help with accelerometer, gyroscope and also adding DualShock 4 digitizer emulation;
- [gl33ntwine](https://github.com/v-atamanenko) for creating [the awesome article](https://gl33ntwine.com/posts/develop-for-vita) for those new to development for the PS Vita;
- [CasualX](https://github.com/CasualX) for [ViGEm client in Rust](https://github.com/CasualX/vigem-client).
