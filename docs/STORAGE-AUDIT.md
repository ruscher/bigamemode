# Storage audit

Where the disk space of a BiGame-mode working tree goes, what may be removed,
and what keeps it from growing back. Measured on the lab laptop's working
tree on 2026-09-28, before anything was removed (`du -sb`, sizes in binary
units).

## Where the 24.7 GiB were

| Path | Size | Kind | Needed? | Recreatable? | Action |
|---|---:|---|---|---|---|
| `bigame-engine/target/debug/examples` | 8.72 GiB | Cargo: 24 example programs with debug info, up to three builds each (`build`, `test`, `clippy` resolve features differently) | no | yes | REGENERATE |
| `bigame-engine/target/debug/incremental` | 6.95 GiB | Cargo incremental sessions (186 directories, 0.4 GiB each for `bigame-core`) | no | yes | REGENERATE |
| `bigame-engine/target/debug/deps` | 5.64 GiB | Cargo: dependencies and test binaries with full debug info | no | yes | REGENERATE |
| `bigame-engine/target/debug/build` | 183 MiB | build-script output | no | yes | REGENERATE |
| `bigame-engine/target/release` | 1.76 GiB | Cargo release build and examples | no | yes | REGENERATE |
| `bigamemode/` (untracked) | 1.29 GiB | an old clone (2.0.0, 2026-09-26) holding a second clone and a makepkg build: `src/` 1.02 GiB (a `target/` and a `cargo-home/`), `pkg/` 20 MiB, `bigame-mode-2.0.0-1-x86_64.pkg.tar` 21 MiB, makepkg's bare mirror 38 MiB, two `.git` of 49 MiB | no | yes | REMOVE |
| `bigame-engine/benchmarks/*/run-*/` raw captures (ignored) | 16.6 MiB | the 2026-09-28 sessions' per-frame profiles, GPU samples, game logs | for re-analysis only | no | MOVE out of the tree |
| `.git` | 48.9 MiB | history of `main` (48.0 MiB, including the raw benchmark data removed in PR #27) plus a local backup tag | yes | from GitHub | KEEP |
| tracked files (665) | 16.4 MiB | code, translations (11.5 MiB), docs and screenshots (1.7 MiB), benchmark summaries | yes | — | KEEP |
| `bigame-engine/scripts/__pycache__`, `locale/pt_BR.po~` | 166 KiB | Python bytecode, a `msgmerge` backup | no | yes | REMOVE |
| `_arms-hybrid/{plain,gamescope}/bigame-mode/games/` | 0 | empty directories the launcher creates when it reads a configuration | no | yes | REMOVE |

```text
Initial size:          24.66 GiB (26 479 649 731 bytes)
Removable:             24.59 GiB (target 23.30 GiB, old clone 1.29 GiB)
Moved out:             16.6 MiB (raw benchmark captures)
Required:              65.3 MiB (tracked files 16.4 MiB + .git 48.9 MiB)
Expected after:        about 65 MiB
Saving:                about 24.6 GiB, 99.7 %
```

**The cause.** 94 % is Cargo's `target/`, and 92 % of that is the debug
profile. The workspace had no `[profile]` settings, so every debug binary
carried full debug info for GTK, zbus, tokio and every other dependency
(`bigame-ui` 249 MiB, `bigame-daemon` 128 MiB, each example about 96 MiB).
Cargo never removes stale artefacts, so every dependency or feature change
left another full set behind. The rest is an old clone with a package
build inside it.

## Looked for and not found in the tree

- Wine/Proton prefixes (`compatdata`, `pfx`, `drive_c`, `user.reg`): none.
  The Steam prefixes are outside the tree, and this cleanup does not touch
  them.
- Shader caches (DXVK, VKD3D, Mesa, NVIDIA): none.
- Third-party binaries (`*.dll`, `nvngx*`, archives, AppImages): none. The
  only tracked binaries are the eight README screenshots in
  `docs/screenshots/`. The only archive, a `.tar.xz` bundled by the
  `gettext-sys` crate, sits inside the old clone's Cargo home.
- `Lossless.dll`: not in the tree. The user's copy lives outside the
  repository and stays untouched.
- Log files other than benchmark captures: none. The application logs to
  journald.

## The installed product

`pacman -Ql bigame-mode` (2.2.0-4): 24.3 MiB installed. It holds two
stripped binaries (`bigame-ui` 13 MiB, `bigame-daemon` 3.8 MiB), the D-Bus,
Polkit and systemd files, icons, 29 catalogues, the README and the licence.
It carries no build output, tests, benchmarks, scripts or caches.

What the application writes at run time (XDG directories, never the
install or source tree):

| Directory | Size | Content | Growth |
|---|---:|---|---|
| `~/.cache/bigame-mode/graphics/optiscaler/` | 494 MiB | OptiScaler 0.9.3 and 0.9.4, each an unpacked release (188–203 MiB) **and** its `.7z` (51–53 MiB) | unbounded: every release ever used stays, and the archive is kept after unpacking although nothing reads it again |
| `~/.cache/bigame-mode/benchmark/` | 0 | MangoHud captures of *Measure the difference* | bounded: emptied before every run |
| `~/.local/state/bigame-mode/` | 132 KiB | install manifests, backups of replaced files, Turbo state | bounded by the number of games |
| `~/.config/bigame-mode/` | 24 KiB | settings and profiles | user data |

## Git

- `git count-objects -vH`: 390 loose objects (5.9 MiB), 43.9 MiB packed, no
  garbage.
- `main` itself is 48 MiB, most of it benchmark data committed before
  PR #27. Rewriting public history is out of scope.
- A local tag, `backup-before-final-cleanup-20260928`, keeps 39 MiB of the
  history from before the attribution rewrite. It is the user's safety net,
  exists only locally, and is kept.
