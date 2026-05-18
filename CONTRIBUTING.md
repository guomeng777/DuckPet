# Contributing

Thank you for considering a contribution to DuckPet.

## Before You Start

- Keep changes focused and easy to review.
- Do not commit generated build output, installers, `node_modules/`, or Rust `target/` directories.
- Be careful with pet assets. Only add artwork, sounds, sprites, or character assets that you created or have permission to redistribute.
- Include attribution for any third-party asset in the pet folder README and, when appropriate, in `NOTICE.md`.

## Development Setup

```powershell
cd tauri-app
npm install
npm run tauri:dev
```

## Build Check

```powershell
cd tauri-app
npm run build
npm run tauri:build
```

If Tauri's local bundler tools are not cached yet, the first package build may need network access.

## Pull Requests

Before opening a pull request:

- Run the relevant build or test command.
- Update documentation when behavior changes.
- Keep generated release artifacts out of the commit.
- Explain any new bundled asset source and redistribution permission.

## Issue Reports

When reporting a bug, include:

- Windows version.
- DuckPet version or commit.
- Whether you installed from a release or ran from source.
- Steps to reproduce.
- Expected behavior and actual behavior.
- Screenshots or logs when useful.
