# Notes to developers

This guide describes how to contribute to this project (and not just this PS Vita HomeBrew) with the least amount of agony.

- [Notes to developers](#notes-to-developers)
  - [1. Beginning](#1-beginning)
    - [1.1 Literature](#11-literature)
    - [1.2 Communities](#12-communities)
    - [1.3 Tools for development](#13-tools-for-development)
  - [2. Notes on the development of the VitaOxiPad](#2-notes-on-the-development-of-the-vitaoxipad)
    - [2.1 General](#21-general)
    - [2.2 Client (`./client`)](#22-client-client)
    - [2.3 Server (`./server`)](#23-server-server)

## 1. Beginning

### 1.1 Literature

- [Getting Started with PSVita Homebrew Development](https://gl33ntwine.com/posts/develop-for-vita);
- [Fixing linking issues with VitaSDK](https://gl33ntwine.com/notes/vita-find-symbol.html);
- [VitaSDK - Development tools for PS Vita](https://vitasdk.org);
- [VitaSDK API](https://docs.vitasdk.org)
- [All about PS Vita hardware](https://www.psdevwiki.com/vita);
- [HENkaku 変革 project](https://henkaku.xyz);
- [taiHEN Quick Start](https://github.com/yifanlu/taiHEN/blob/master/USAGE.md);
- [Using the VitaSDK/DolceSDK with Visual Studio Code](https://forum.devchroma.nl/index.php?topic=139.0).

### 1.2 Communities

- [Vita Nuova](https://discord.com/invite/PyCaBx9) Discord server;
- [HENkaku](https://discord.com/invite/m7MwpKA) Discord server.

### 1.3 Tools for development

- [Vitadev Package manager](https://github.com/vitasdk/vdpm);
- [A well-running Vitacompanion fork](https://github.com/devnoname120/vitacompanion);
- [Python3 vita-parse-core](https://github.com/isage/vita-parse-core) to retrieve information from the PS Vita error dump;
- Various development tools [PSVita-RE-tools](https://github.com/TeamFAPS/PSVita-RE-tools),
   including [PrincessLog](https://github.com/TeamFAPS/PSVita-RE-tools?tab=readme-ov-file#princesslog---by-princess-of-sleeping) for debug.

## 2. Notes on the development of the VitaOxiPad

### 2.1 General

- Use automatic code formatting for alignment and indentation (VSCode, Pre-Commit, etc).

### 2.2 Client (`./client`)

**Cheat sheets:**

- Compile and run: `cargo build --release --bin cli && ./target/release/cli $VITA_IP`;
- Clean cargo packages: `cargo clean`;
- Upgrade all cargo dependencies: `cargo install cargo-edit` and `cargo upgrade`.

**Notes:**

- To check the data sent to `uinput` it is convenient to use `evtest` with [evtest-qt](https://github.com/Grumbel/evtest-qt).

### 2.3 Server (`./server`)

**Cheat sheets:**

- Generate build folder: `cmake -S server -B build`;
- Compile: `cmake --build build -j`;
- Send to PS Vita: `cmake --build build --target send`;
- Download and parse error dump: `export PARSECOREPATH=/vita-parse-core/main.py` and `cmake --build build --target dump`.

**Notes:**

- If you change the code in a kernel module, [update its version](https://github.com/DvaMishkiLapa/VitaOxiPad/blob/63f484d6a2899df04f94252086461702db1f8893/server/module/include/kctrl-kernel.h#L10).
   This will help inform users after the update that they should reboot, as the old module may still be in memory;
- If you modify Cmake instructions, don't be lazy to do a complete cleanup of the project (for the lazy, you can just delete the build folder).
   This will help you avoid a lot of non-obvious problems;
- `cmake --build build -j && cmake --build build --target send` a quick way to send changes to the PS Vita.
   Just don't forget to specify the `VITA_IP` variable before generating build instructions via `cmake -S server -B build`;
- Keep in mind that `cmake --build build --target send` will only send the module and eboot executable files. The same is true for `PARSECOREPATH`.
