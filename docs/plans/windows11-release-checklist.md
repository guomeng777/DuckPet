# Windows 11 Release Checklist

This checklist applies to the DuckPet Tauri v2 Windows build.

## Build Artifacts

- Run `npm.cmd run build` from `tauri-app`.
- Run `npm.cmd run tauri build` from `tauri-app`.
- Confirm the release executable exists at `tauri-app/src-tauri/target/release/tauri-app.exe`.
- Confirm the NSIS installer exists under `tauri-app/src-tauri/target/release/bundle/nsis/`.
- Confirm Tauri build tools are cached under `tauri-app/src-tauri/target/.tauri/` when `bundle.useLocalToolsDir` is enabled.
- If NSIS packaging is blocked by tool download restrictions, confirm the fallback portable zip exists under `tauri-app/src-tauri/target/release/bundle/portable/` and mark installer verification as blocked.
- Confirm the packaged app can load bundled `Pets` resources after installation.
- Confirm `tauri-app/README.md` documents the exact development, build, run, and package commands.

## Windows 11 Smoke Tests

- Install the generated NSIS installer on a clean Windows 11 user profile.
- Launch DuckPet from the installed shortcut or install directory.
- Confirm the settings window opens.
- Confirm at least one bundled pet loads from XML and renders on the desktop.
- Confirm app startup does not require the legacy C# executable or WinForms runtime.
- Confirm app exit closes all DuckPet windows and processes.

## Multi-Monitor Tests

- Start DuckPet with one monitor connected.
- Start DuckPet with two or more monitors connected.
- Move the pet across monitor boundaries.
- Verify screen bounds, work area clamping, and gravity behavior on each monitor.
- Disconnect or disable a secondary monitor while the app is running and verify the pet remains recoverable.

## DPI Scaling Tests

- Test at 100 percent scaling.
- Test at 125 percent scaling.
- Test at 150 percent scaling.
- Test with mixed-DPI monitors.
- Verify pet position, sprite size, hit testing, and settings window layout remain usable.
- Verify no text clipping in settings controls.

## Transparent Window Tests

- Confirm the pet window background is transparent.
- Confirm sprite edges do not show opaque artifacts.
- Confirm always-on-top behavior works.
- Confirm click-through can be enabled and disabled.
- Confirm click-through does not permanently block access to settings or exit controls.

## Interaction Tests

- Drag the pet and release it.
- Open the pet context menu.
- Toggle always-on-top.
- Toggle click-through.
- Trigger close/exit from the available UI.
- Confirm behavior remains stable when multiple pets are active if multi-pet support is enabled for the build.

## Antivirus And Signing Risk

- Code signing is not configured for the current build.
- Treat the installer and executable as unsigned artifacts.
- Expect possible Windows SmartScreen warnings.
- Expect possible antivirus reputation warnings for fresh unsigned binaries.
- Do not publish as a signed release until a valid certificate and signing pipeline are configured.
- Before public distribution, scan the installer with Windows Defender and at least one independent malware scanning service.

## Known Limitations

- The MVP does not include the legacy editor.
- Online pet download and update flows are not included.
- The old WinForms tray/menu model has been replaced by Tauri windows and frontend controls.
- Some low-level Win32 window behavior may differ from the original C# implementation.
- WebView2 installation is skipped because Windows 11 normally includes WebView2 Runtime.
- Packaged resource loading must be validated from the installed app, not only from development mode.
- The portable zip is only a smoke-test artifact when NSIS/MSI packaging is blocked; it is not a signed installer.

## Release Decision

- All required build artifacts exist.
- Windows 11 smoke tests pass.
- Multi-monitor behavior is acceptable.
- DPI scaling behavior is acceptable.
- Transparent and click-through windows behave as expected.
- Unsigned binary risk is explicitly communicated.
- Known limitations are documented in release notes.
- NSIS/MSI installer generation is either completed or explicitly marked blocked with the portable artifact path.
