# DuckPet

![DuckPet overview](docs/assets/duckpet-info.png)

<p align="center">
  <img alt="Windows 11" src="https://img.shields.io/badge/Windows-11-0078D4?style=for-the-badge&logo=windows&logoColor=white">
  <img alt="Tauri 2" src="https://img.shields.io/badge/Tauri-2.0-24C8DB?style=for-the-badge&logo=tauri&logoColor=white">
  <img alt="React" src="https://img.shields.io/badge/React-19-61DAFB?style=for-the-badge&logo=react&logoColor=0B1220">
  <img alt="TypeScript" src="https://img.shields.io/badge/TypeScript-5-3178C6?style=for-the-badge&logo=typescript&logoColor=white">
  <img alt="Rust" src="https://img.shields.io/badge/Rust-Stable-B7410E?style=for-the-badge&logo=rust&logoColor=white">
  <img alt="License" src="https://img.shields.io/badge/License-MIT-2EA44F?style=for-the-badge">
  <img alt="Desktop Pet" src="https://img.shields.io/badge/Desktop%20Pet-Cozy%20%26%20Cute-F7B955?style=for-the-badge">
</p>

DuckPet is a Windows desktop pet app built with Tauri, React, TypeScript, and Rust. It displays animated pets on the desktop, lets you choose bundled pet definitions, and keeps the legacy eSheep-style XML animation format available for experimentation.

This repository contains the modern Tauri app under `tauri-app/` plus legacy pet resources, manuals, tools, and migration notes.

## Features

- Windows desktop pet windows powered by Tauri WebView2.
- Pet selection from bundled XML animation definitions.
- Multiple pet instances, with start, pause, resume, spawn, close, and close-all controls.
- Optional window collision behavior.
- Optional click-through behavior for pet windows.
- Audio mute/unmute control.
- Bundled legacy `Pets/` resource folder for sprites, animation XML, icons, and sounds.

## Download

Installers are published from GitHub Releases, not committed to the repository.

For a local build, the current Windows installer output is generated at:

```text
tauri-app/src-tauri/target/release/bundle/nsis/
```

The current generated installer name is:

```text
DuckPet_0.1.0_x64-setup.exe
```

The installer is currently unsigned. Windows SmartScreen or antivirus software may warn about unsigned binaries.

## Requirements

- Windows 11
- Node.js and npm
- Rust stable MSVC toolchain
- Visual Studio Build Tools with MSVC and Windows SDK
- Microsoft Edge WebView2 Runtime

Windows 11 normally includes WebView2 Runtime. The current Tauri bundle config skips installing WebView2.

## Run From Source

```powershell
cd tauri-app
npm install
npm run tauri:dev
```

## Build

```powershell
cd tauri-app
npm install
npm run tauri:build
```

Frontend build output:

```text
tauri-app/dist/
```

Windows installer output:

```text
tauri-app/src-tauri/target/release/bundle/nsis/
```

Portable bundle output, when generated:

```text
tauri-app/src-tauri/target/release/bundle/portable/
```

## Project Layout

```text
.
|-- Pets/                 # Bundled pet XML, sprites, icons, and sounds
|-- Resources/            # Shared schemas and resources
|-- Manual/               # Legacy documentation
|-- Tools/                # Legacy pet editor and supporting tools
|-- docs/                 # Migration plans and project notes
|-- tauri-app/            # Tauri 2 + React + TypeScript desktop app
`-- Readme.md             # This file
```

## Third-Party Assets

The application code is licensed under the repository license, but bundled pet artwork, sounds, character names, and sprites may have separate ownership or licensing terms.

Some bundled pets reference third-party or fan-art sources. Before redistributing a binary release, review `NOTICE.md` and decide whether to keep those assets, remove them, or ship them as an optional asset pack.

## License

Application source code is released under the MIT License. See `LICENSE`.

Third-party assets are not automatically relicensed by this repository. See `NOTICE.md` for attribution and asset notes.

## Contributing

Contributions are welcome. Please read `CONTRIBUTING.md` before opening issues or pull requests.

## Security

Please report security issues privately using the process in `SECURITY.md`.
