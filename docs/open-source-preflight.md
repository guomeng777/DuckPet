# DuckPet open source preflight

Date: 2026-05-18

## Current state

- The project root is not a Git repository yet.
- A Tauri app exists under `tauri-app/`.
- Windows release artifacts already exist under `tauri-app/src-tauri/target/release/bundle/`.
- No root `LICENSE`, `NOTICE`, or `COPYING` file was found.
- No `.github/` project templates or workflows were found.

## Do not commit

These paths are generated dependencies or build outputs and should be ignored before the first commit:

- `tauri-app/node_modules/`
- `tauri-app/dist/`
- `tauri-app/src-tauri/target/`
- `tauri-app/vite-dev.log`
- `tauri-app/vite-dev.err.log`

Release installers should be attached to GitHub Releases instead of committed:

- `tauri-app/src-tauri/target/release/bundle/nsis/DuckPet_0.1.0_x64-setup.exe`
- `tauri-app/src-tauri/target/release/bundle/portable/DuckPet-0.1.0-windows-x64-portable.zip`

## Files worth keeping

- `tauri-app/package-lock.json`
- `tauri-app/src-tauri/Cargo.lock`
- `Pets/`
- `Resources/`
- `Manual/`
- `docs/`
- `Tools/`

## License and attribution gaps

Add a clear root license before publishing. MIT is a good default for a permissive project, while GPL-3.0 is better if derivative works should remain open source.

Add `NOTICE.md` or `CREDITS.md` for upstream project and third-party asset attribution. The current root `Readme.md` still references the original Desktop Pet/eSheep project and its download links, so it should be rewritten for DuckPet before publication.

High-risk or unclear asset folders:

- `Pets/pikachu/`: Pokemon character asset, source link only.
- `Pets/mareep/`: Pokemon character asset, source link only.
- `Pets/shiny_sylveon/`: Pokemon character asset, source link only.
- `Pets/ssj-goku/`: Dragon Ball character asset, sprite source link only.
- `Pets/bbunny/`: Buster Bunny asset, sprite source link only.

Recommendation: publish the app source first with only assets you are confident are redistributable, or clearly separate third-party fan assets into an optional pack with explicit attribution and legal disclaimer.

## Secret scan result

No obvious GitHub tokens, OpenAI keys, AWS access keys, or private-key blocks were found in source files outside generated dependency/build directories.

The broad keyword scan matched many false positives in:

- `Pets/*/animations.xml` embedded base64 image/audio data.
- `docs/scripts/jquery-1.11.0.min.js`.
- `Tools/PetEditor/Resources/lite.render.js`.
- `tauri-app/package-lock.json`.

## Large tracked-file candidates

These source-side files are large enough to check intentionally before upload:

- `Pets/esheep_ani.gif` (~2.8 MB)
- `Pets/*_sheep/animations.xml` (~1.1 MB each)
- `Tools/PetEditor/Resources/lite.render.js` (~1.4 MB)

They are not automatically wrong, but they will make the repository heavier.

## Required cleanup before first GitHub push

1. Update `.gitignore` for Tauri, Node, Rust, logs, and release archives.
2. Rewrite root README for DuckPet, not the upstream Desktop Pet project.
3. Add a root `LICENSE`.
4. Add a root `NOTICE.md` or `CREDITS.md`.
5. Decide whether to remove, keep, or separate third-party character assets.
6. Run `git init`, `git add .`, and inspect `git status` before committing.
7. Upload installers through GitHub Releases, not Git history.

