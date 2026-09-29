# Storage cleanup report

The cleanup of the lab laptop's working tree on 2026-09-28/29, following
[STORAGE-AUDIT.md](STORAGE-AUDIT.md). Sizes from `du -sb`.

```text
BEFORE     24.66 GiB  (26 479 649 731 bytes)
AFTER      65.3 MiB   (68 524 179 bytes)
REMOVED    24.60 GiB
REDUCTION  99.74 %
```

## What was removed or moved

| What | Size | How |
|---|---:|---|
| `bigame-engine/target/` | 23.30 GiB | `cargo clean`; rebuilt from zero afterwards |
| `bigamemode/`, an old clone (2.0.0) with a second clone and a makepkg build inside | 1.29 GiB | removed after checking that its tree equals `a3c4fde` on `main`, that every one of its commits is on `main` under the same subject, and that it had no local changes or stashes |
| raw captures of the 2026-09-28 benchmark sessions (153 files) | 16.6 MiB | moved, not deleted, to `~/.local/share/bigame-mode/benchmarks/raw/` with their session layout |
| Python bytecode, a `msgmerge` backup, two empty directories | 166 KiB | removed |

Nothing tracked by git was removed. Nothing outside the tree was touched
except the application's own download cache (below): not the Steam library,
not the Wine prefixes, not the user's `Lossless.dll`.

## What keeps it from growing back

- **Cargo profiles** (`bigame-engine/Cargo.toml`). Debug builds carry debug
  info only for the workspace's own crates, split into their object files.
  One build, test and clippy pass from clean now leaves 2.57 GiB instead of
  5.85 GiB, in 156 s instead of 191 s; `bigame-ui` is 63 MiB instead of
  249 MiB. Backtraces keep file and line, and gdb still stops in
  `bigame_core` with its arguments and source. After this whole session of
  changes, tests and release lints, `target/` held 4.0 GiB; before, it had
  accumulated 23.3 GiB.
- **`bigame-engine/scripts/clean-dev.sh`**:
  - `--analyze` (the default) shows what the tree holds;
  - `--safe` removes only what a build recreates;
  - `--deep` also moves benchmark raw captures out of the tree, keeping the
    newest 20 sessions (`RAW_KEEP`), after listing everything and asking.

  Only git-ignored files are candidates; nested clones are reported, never
  removed. It was tested in a throwaway clone with a fake file of every
  category, then used for the final `--safe` pass here.
- **`.gitignore`** now covers every `*.pkg.tar.*`, source packages, crash
  dumps and profiler output. No tracked file matches the new rules.
- **The application's OptiScaler cache**
  (`~/.cache/bigame-mode/graphics/optiscaler/`):
  - the downloaded `.7z` is removed once unpacked;
  - after Apply, Update and Restore, a release no game uses (installed, or
    kept for Go back) is removed, unless it was fetched in the last day;
  - Repair and Go back download a missing release again and check its hash;
  - release and asset names must be plain file names before they become
    paths.

  On this machine the cache went from 494 MiB to 203 MiB (0.9.3 unused, both
  archives), and `graphics_diagnose` still reports the Shadow of the Tomb
  Raider install "installed and intact".

  The application's other stores were already bounded: *Measure the
  difference* empties its capture folder before every run, logs go to
  journald, and state is a few KiB per game. A cache button in Settings was
  not added, because what the cache keeps now is what installs need.

## The package

- **Release profile:** `lto = true`, `codegen-units = 1`.
- **Stripped binaries:** `bigame-ui` 14 → 9.6 MiB, `bigame-daemon` 3.9 →
  2.6 MiB.
- **Installed size:** 24.3 → 19.7 MiB, of which 7.8 MiB is the 29
  translation catalogues.
- **Contents:** unchanged — two binaries, D-Bus, Polkit and systemd files,
  icons, catalogues, README, licence.
- **Build cost:** the PKGBUILD's build and check take 467 s instead of 243 s
  on this laptop.
- **Kept as they were:** panics still unwind (`panic = "abort"` would let
  one failing D-Bus call end the daemon), and makepkg does the stripping.

## Rebuilt and checked from zero

- **makepkg from a fresh clone:** the unmodified PKGBUILD, with its source
  pointed at a fresh clone of the branch, an empty Cargo home and no
  `target/`, built the package in 509 s. `cargo fetch --locked`, then
  `build --frozen`, then all 739 tests passed.
- **In the tree:** `cargo test --workspace` passed 739, and clippy
  `-D warnings` is clean in debug and release. Shellcheck at warning level
  is clean, and the translation template is up to date.
- **The new package's binary on this machine:**
  - `bigame-ui --diagnostics` read Turbo on, falcond active and owned, the
    More FPS preset, the GTX 1050 Ti as the GPU games render on, 16
    sched-ext schedulers, and a Booster plan;
  - the window opened on the game library;
  - `health` reported the D-Bus helper reachable;
  - `turbo status`, `detect`, `launch_plan` (PRIME offload), `lsfg` (layer
    and DLL ready) and `graphics_capabilities` (OptiScaler available, no
    DLSS on the GTX) behaved as before.
- **Installed:** the package built the same way (as 2.2.0-4.1, so it
  replaces 2.2.0-4) was installed with pacman. The old helper was stopped by
  the install script, and D-Bus started the new one on the next call. Turbo
  went off and on again through it, Polkit and falcond, with the More FPS
  preset; `health` reported the helper reachable, and the kernel log showed
  no GPU errors.

## The 20 largest directories left

| Size | Directory |
|---:|---|
| 51 MiB | `.git` (45 MiB packed) |
| 12 MiB | `locale` |
| 4.9 MiB | `bigame-engine` |
| 2.0 MiB | `bigame-engine/bigame-core` (1.9 MiB `src`) |
| 1.9 MiB | `bigame-engine/benchmarks` (reports and summaries) |
| 1.7 MiB | `docs` (1.7 MiB `screenshots`) |
| 956 KiB | `bigame-engine/bigame-ui` (536 KiB `src/views`, 288 KiB `src/widgets`) |
| 500 KiB | `bigame-engine/bigame-core/src/graphics` |
| 44 KiB | `data` |
| 40 KiB | `usr` |
| 36 KiB | `style` |
| 4 KiB | `tests` |

Below these, only `.git`'s object fan-out directories (under 300 KiB each).
