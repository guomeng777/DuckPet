# DuckPet Tauri App

DuckPet is the Windows 11 Tauri v2 migration target for the legacy DesktopPet/eSheep C# project. This app uses React, TypeScript, Vite, and Rust/Tauri.

The existing C# project remains in place. This directory only contains the new Tauri application.

## Requirements

- Windows 11
- Node.js and npm
- Rust stable MSVC toolchain
- Visual Studio Build Tools with MSVC and Windows SDK
- Microsoft Edge WebView2 Runtime

Windows 11 normally includes WebView2 Runtime. The current Tauri bundle config skips installing WebView2.

## Commands

Install dependencies:

```powershell
npm install
```

Run the Vite dev server only:

```powershell
npm.cmd run dev
```

Build the frontend only:

```powershell
npm.cmd run build
```

Run the desktop app in development mode:

```powershell
npm.cmd run tauri dev
```

Build the Windows bundle:

```powershell
npm.cmd run tauri build
```

Equivalent npm script aliases are also available:

```powershell
npm.cmd run tauri:dev
npm.cmd run tauri:build
```

## Output Locations

Frontend build output:

```text
tauri-app/dist/
```

Tauri release executable:

```text
tauri-app/src-tauri/target/release/tauri-app.exe
```

Windows NSIS installer output:

```text
tauri-app/src-tauri/target/release/bundle/nsis/
```

The Tauri config uses project-local build tool caching:

```text
tauri-app/src-tauri/target/.tauri/
```

Portable smoke-test bundle output, when generated manually from the Tauri release executable and bundled resources:

```text
tauri-app/src-tauri/target/release/bundle/portable/DuckPet-0.1.0-windows-x64-portable.zip
```

## Packaged Assets

The legacy `Pets` directory is bundled as a Tauri resource and is required for pet XML definitions, sprites, sounds, and animation data.

Configured resource mapping:

```text
../../Pets -> Pets
```

## Signing Status

Code signing is not configured. The generated installer and executable are unsigned and may trigger Windows SmartScreen or antivirus warnings.

Do not describe a release artifact as signed unless a valid signing certificate and signing pipeline are configured.

## Known Limitations

- The app is still an MVP migration and does not include the legacy pet editor.
- Online pet download/update features are not included.
- Some legacy WinForms and Win32 window behavior has been rewritten and still needs packaged-app smoke testing.
- Click-through mode disables direct pet mouse interaction until it is reset from settings or the app is restarted.
- Final installed-path resource behavior must be verified from the generated installer on Windows 11.
- If the NSIS tool package is not already cached in `tauri-app/src-tauri/target/.tauri/`, `npm.cmd run tauri build` needs network access to download Tauri's NSIS bundler.
