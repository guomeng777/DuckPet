# Tauri Migration Notes

Date: 2026-05-15

Scope: Task 1 repository migration assessment for rebuilding the current DesktopPet/eSheep C# project as a Windows 11 + Tauri + React + TypeScript app. This note is documentation only; no existing source files were changed.

## Files Reviewed

- `Readme.md`
- `Download.md`
- `src/dotNet/Program.cs`
- `src/dotNet/FormPet.cs`
- `src/dotNet/Animations.cs`
- `src/dotNet/Xml.cs`
- `src/Resources/animations.xsd`
- `Pets/esheep64/animations.xml`
- Supporting context checked: `src/dotNet/StartUp.cs`, `src/dotNet/FormPet.Designer.cs`, `Pets/esheep64/README.md`

## Current C# Runtime Flow

1. `Program.Main` starts as an STA WinForms application, enables visual styles, loads embedded portable assemblies (`NAudio.dll`, `Newtonsoft.Json.dll`), registers `AssemblyResolve`, and enforces a maximum of two app instances through named mutexes.
2. Startup arguments can request an external XML source (`localxml=`, `webxml=`, `install=`). If present, `Program.MyData.LoadXML()` is loaded into the application data store before runtime initialization.
3. `ProcessIcon` creates the tray icon, then `StartUp` becomes the main runtime coordinator. `Application.Run()` keeps the WinForms message loop alive without a traditional main window.
4. `StartUp` constructs `Xml` with the configured scale factor, constructs `Animations`, reads the current XML through `Xml.ReadXML()`, falls back to embedded default XML on parse/load failure, sets the tray icon metadata, and starts a one-second timer before spawning the first pet.
5. `Xml.ReadXML()` deserializes the XML into generated XML data types, stores base64 image/icon data, decodes the sprite sheet and icon, calculates sprite tile dimensions, and builds a per-frame bitmap list.
6. `Xml.LoadAnimations()` translates the XML document into runtime dictionaries: animations, spawns, child animations, and sounds. It also records special animation names (`fall`, `drag`, `kill`, `sync`).
7. `StartUp.AddSheep()` creates one `FormPet`, copies every decoded sprite into its `ImageList`, sizes the form to one tile, calls `FormPet.Play(true)`, and tracks active pets up to `MAX_SHEEPS = 16`.
8. `FormPet.Play()` chooses a weighted spawn, evaluates spawn `x` and `y` expressions against the current screen/work area, mirrors horizontal placement when flipped, selects the spawn's next animation, shows the transparent topmost form, and starts the pet timer.
9. `FormPet.Timer1_Tick()` drives the runtime. Every tick calls `NextStep()`, increments the animation step, and re-enables the timer using the current animation interval.
10. `FormPet.NextStep()` selects the current sprite frame, interpolates interval/opacity/offset/movement between `start` and `end`, applies drag behavior, checks screen borders, taskbar/work-area floor, window collision, gravity, sequence completion, `flip` action, kill fade-out, and then applies the new form position.

## XML Data Model

The legacy format is a single namespaced root:

```xml
<animations xmlns="https://esheep.petrucci.ch/">
```

Core top-level sections:

- `header`: required metadata fields `author`, `title`, `petname`, `version`, `info`, `application`, and `icon` as base64 CDATA.
- `image`: required sprite-sheet fields `tilesx`, `tilesy`, `png` as base64 CDATA, and `transparency` color key.
- `spawns`: one or more `spawn` nodes with `id`, `probability`, `x`, `y`, and `next`.
- `animations`: one or more `animation` nodes with `id`, `name`, `start`, optional `end`, required `sequence`, optional `border`, optional `gravity`.
- `childs`: optional child pet spawn definitions keyed by `animationid`.
- `sounds`: optional per-animation sound definitions keyed by `animationid`.

`Pets/esheep64/animations.xml` observed values:

- Header: author `Adriano`, title `eSheep 64bit`, petname `eSheep`, version `1.8`.
- Image grid: `tilesx = 16`, `tilesy = 11`, therefore 176 possible tile slots.
- Runtime data: 4 spawns, 54 animations, 3 child definitions, 0 sounds.
- Example spawn expressions include `screenW+10`, `areaH-imageH`, `random*(screenW-imageW-50)/100+25`, and `areaH/2-(randS*areaH/2)/120-imageH`.

Animation node details:

- `start` and `end` use the shared `step` shape: `x`, `y`, optional `offsety`, optional `opacity`, and `interval`.
- `sequence` contains ordered `frame` indexes, optional `action`, and zero or more `next` transitions. Attributes: `repeat` as string expression and `repeatfrom` as integer.
- `border` and `gravity` contain `next` transitions used when collision or falling conditions are detected.
- `next` has integer text content, optional weighted `probability`, and optional `only`.
- `only` supports `none`, `taskbar`, `window`, `horizontal`, `horizontal+`, and `vertical`.
- `action` currently matters for `flip`, which mirrors sprite direction and future horizontal movement.

Expression compatibility is important. The C# implementation stores expressions as strings and evaluates them later when dynamic or screen-dependent values are present. Supported variables in the current code include:

- Screen/work area: `screenW`, `screenH`, `areaW`, `areaH`
- Sprite/parent: `imageW`, `imageH`, `imageX`, `imageY`
- Random: `random`, `randS`
- Scale: `scale`

The old evaluator is `DataTable.Compute`, so some existing XML expressions also use .NET-style syntax such as `Convert(screenW/2,System.Int32)%30`.

## WinForms And Win32 Dependency Points

WinForms UI/runtime dependencies:

- `Application.Run()` and the WinForms message loop are the process lifetime.
- `NotifyIcon`/tray behavior is managed through `ProcessIcon`.
- Each pet is a separate `FormPet` window.
- Rendering uses `PictureBox`, `ImageList`, `Bitmap`, `Graphics`, and `System.Drawing`.
- Animation timing uses `System.Windows.Forms.Timer`.
- Pet mouse interactions use `MouseDown`, `MouseUp`, `DoubleClick`, `Click`, `DragEnter`, and `DragDrop`.
- Debug/option UI uses additional WinForms forms and context menus.

Window style and desktop behavior dependencies:

- `FormPet.CreateParams` adds `WS_EX_TOOLWINDOW` to hide pet windows from Alt-Tab.
- `WS_EX_TOPMOST` keeps the pet above normal windows.
- `WS_EX_LAYERED` is used for layered window behavior/performance.
- `WS_EX_NOACTIVATE` is used for child pet forms.
- `ShowWithoutActivation` prevents focus stealing on show.
- `TopMost` is toggled during drag/fullscreen behavior.
- `Screen.AllScreens`, `Screen.Bounds`, and `Screen.WorkingArea` provide monitor and taskbar-aware work area data.

Direct Win32 calls in `FormPet.NativeMethods`:

- `GetWindowRect`
- `EnumWindows`
- `IsWindowVisible`
- `GetWindowText`
- `GetTitleBarInfo`
- `ShowWindow`
- `SetForegroundWindow`
- `GetWindow`
- `GetTopWindow`
- `GetForegroundWindow`
- `FindWindowEx`

Behavior implemented through those calls:

- Detect visible desktop windows below the pet while falling.
- Ignore windows without usable title/title-bar information.
- Track whether the pet is walking on a particular window.
- Follow a window when it moves or resizes.
- Check Z-order so the pet does not stand on a covered window.
- Detect fullscreen foreground windows and temporarily disable topmost behavior.
- Optionally bring the detected window to foreground.
- Locate taskbar thumbnail window class as part of taskbar-related behavior.

## Reusable Assets

Directly reusable:

- `Pets/*/animations.xml`: legacy pet manifests. All scanned pet folders currently contain `animations.xml`.
- `Pets/*/icon.png`: pet listing/selection icons. All scanned pet folders currently contain `icon.png`.
- Base64 sprite sheet data inside each `animations.xml`.
- Base64 icon data inside each `animations.xml`.
- `src/Resources/animations.xsd`: compatibility contract for the XML parser.
- Documentation assets and existing pet README files.

Reusable as behavioral references, not as code:

- Animation state structures and weighted transition rules from `Animations.cs`.
- XML field mapping and fallback behavior from `Xml.cs`.
- Window collision and gravity heuristics from `FormPet.cs`.
- Tray and multi-pet coordination ideas from `StartUp.cs` and `ProcessIcon.cs`.

## Modules That Must Be Rewritten

Must be rewritten in Rust/Tauri:

- Native window creation/configuration for transparent, frameless, topmost pet windows.
- Win32 platform adapter for monitors, work area, taskbar/floor, foreground/fullscreen detection, visible window enumeration, Z-order checks, and click-through/topmost flags.
- XML parser for the legacy namespaced `animations.xml` format.
- Safe expression evaluator. The current `DataTable.Compute` approach should not be carried over because it is too broad for untrusted XML-like data.
- Animation runtime state machine: spawn selection, frame sequencing, repeat/repeatfrom, movement interpolation, opacity/offset, transitions, gravity, borders, flip, child spawning, kill/drag/fall/sync special names.
- Sprite sheet decoding and tile selection pipeline.
- Audio playback for optional XML sounds.
- Runtime command API exposed to React via Tauri invokes.
- App state storage and settings migration.
- Packaging, autostart, and Windows installer/portable distribution.

Must be rewritten in React/TypeScript:

- Settings window and pet selector.
- Runtime controls: start, pause, resume, close, spawn another pet.
- Debug panel for current animation, frame index, position, platform info, and parser errors.
- Canvas-based pet rendering surface.
- Context menu and drag interactions that forward state changes to Rust.

Can be postponed or reduced:

- Offline pet editor.
- Web pet download flow.
- Perfect parity with all old window-collision edge cases.
- Microsoft Store/UWP integration.
- Full compatibility with every old `DataTable.Compute` expression if rare expressions are not needed by the MVP.

## MVP Scope

Minimum viable migration milestone:

1. Create a Tauri v2 + React + TypeScript app under `tauri-app/`.
2. Open a normal settings window.
3. Open one transparent, frameless, topmost pet window on Windows 11.
4. Load `Pets/esheep64/animations.xml` without manually converting it.
5. Parse header, image grid, spawns, animations, sequence frames, sequence transitions, border transitions, and gravity transitions.
6. Decode the base64 sprite sheet and render tiles on a canvas.
7. Evaluate the expressions needed by eSheep's initial spawn and basic walking/falling path.
8. Spawn one eSheep, render animated frames over time, and move it according to XML `start`/`end` values.
9. Treat the current work area bottom as the first floor/taskbar boundary.
10. Support sequence completion, weighted `next`, basic border transition, basic gravity transition, and `flip`.
11. Show enough debug information in settings to inspect parser/runtime state.

Out of MVP:

- Multi-pet parity beyond a small controlled limit.
- Full child animation support.
- Sound playback.
- Complex window collision parity.
- Context menu parity.
- Drag/drop XML replacement.
- Online XML loading.
- Full options/autostart/tray feature parity.
- Signed release artifacts.

## Suggested Next Step

Proceed with Task 2: scaffold the new `tauri-app/` as a Tauri v2 + React + TypeScript + Vite app without moving or deleting the existing C# project. Keep the first build empty and verify the frontend can build before adding native window behavior.
