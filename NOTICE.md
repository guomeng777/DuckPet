# Notices and Attribution

This project includes application source code, documentation, legacy tools, and bundled pet assets. The repository license applies to the DuckPet application source code unless a file or directory states a different license.

Bundled pet artwork, sounds, sprites, character names, and animation data may have separate ownership or licensing terms. Do not assume third-party assets are covered by the MIT License.

## Upstream Project

DuckPet is a Tauri migration inspired by the legacy Desktop Pet/eSheep project.

- Upstream project: https://github.com/Adrianotiger/desktopPet
- Related web project: https://github.com/Adrianotiger/web-esheep

The legacy root README previously referenced the upstream Desktop Pet project and its download links. DuckPet keeps legacy resources and documentation for compatibility and migration work.

## Bundled Pet Assets

The `Pets/` directory contains XML animation definitions, sprites, icons, and sounds. Some pets include attribution in their own `README.md` files.

Known asset folders requiring review before public binary redistribution:

- `Pets/pikachu/`: Pokemon character asset. The local README references https://eeveeexpo.com/resources/.
- `Pets/mareep/`: Pokemon character asset. The local README references https://eeveeexpo.com/resources/.
- `Pets/shiny_sylveon/`: Pokemon character asset. The local README references https://eeveeexpo.com/resources/.
- `Pets/ssj-goku/`: Dragon Ball character asset. The local README credits RedSparr0w and references spritedatabase.net.
- `Pets/bbunny/`: Buster Bunny character asset. The local README references Spritedatabase.net and Daisy of the Wolves.

These notes are attribution pointers, not legal clearance. If you publish official builds, either verify redistribution rights for each asset, remove unclear assets, or distribute them separately as optional community assets.

## Third-Party Dependencies

The Tauri app uses open source dependencies managed through npm and Cargo. See:

- `tauri-app/package.json`
- `tauri-app/package-lock.json`
- `tauri-app/src-tauri/Cargo.toml`
- `tauri-app/src-tauri/Cargo.lock`

Dependency licenses remain governed by their respective projects.
