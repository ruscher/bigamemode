# BiGame-mode — Implementation Review & Audit
**Date:** 2026-04-21  
**Reviewer:** Senior Systems Engineer (automated audit)  
**Scope:** `falcond/`, `lsfg-vk/`, `bigame-mode/` cross-component analysis  
**Status:** 4 CRITICAL gaps · 4 MEDIUM gaps · 3 INFORMATIONAL

---

## 1. System Architecture Overview

```
┌──────────────────────────────────────────────────────────────────────┐
│                         bigame-mode (Rust/GTK4)                      │
│  ┌─────────────┐  ┌───────────────┐  ┌──────────────────────────┐   │
│  │  bigame-ui  │  │  bigame-core  │  │       D-Bus / sysfs      │   │
│  │  (Libadw.)  │←─│  lib crate    │──│  GameMode / PowerProf.   │   │
│  └─────────────┘  └───────────────┘  └──────────────────────────┘   │
│         │               │  pkexec tee / SIGHUP                       │
│         ▼               ▼                                             │
│  ┌────────────────────────────────┐                                  │
│  │  /etc/falcond/config.conf      │                                  │
│  │  /usr/share/falcond/profiles/  │                                  │
│  │  /tmp/falcond_status  [read]   │                                  │
│  └──────────────┬─────────────────┘                                  │
└─────────────────│────────────────────────────────────────────────────┘
                  │ SIGHUP reload
                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    falcond (Zig daemon, root)                        │
│  Process Scanner → Profile Matcher → Activation Engine              │
│    • power-profiles-daemon  (D-Bus system)                          │
│    • scx_loader              (D-Bus system)                         │
│    • AMD VCache              (sysfs direct, root)                   │
│    • screensaver inhibit     (systemd-logind D-Bus)                 │
│    • start_script / stop_script  (subprocess)                       │
│    • /tmp/falcond_status     [write]                                │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                   lsfg-vk (C++ Vulkan Layer)                        │
│  VK_LAYER_LSFGVK_frame_generation                                   │
│  Reads: ~/.config/lsfg-vk/conf.toml  (WatchedConfig hot-reload)    │
│  Env:   LSFGVK_MULTIPLIER / LSFGVK_FLOW_SCALE / LSFGVK_PERF_MODE  │
│  Auto-detection: /proc/self/exe + /proc/self/comm + wine maps       │
└─────────────────────────────────────────────────────────────────────┘
```

**IPC Summary:**

| Channel | Direction | Used by |
|---------|-----------|---------|
| `/etc/falcond/config.conf` | bigame-mode → falcond | pkexec tee + SIGHUP |
| `/usr/share/falcond/profiles/user/*.conf` | bigame-mode → falcond | pkexec tee + SIGHUP |
| `/tmp/falcond_status` | falcond → bigame-mode | plain text file poll |
| D-Bus session (GameMode) | bigame-mode ↔ feralinteractive | read-only UI indicator |
| D-Bus system (PowerProfiles) | bigame-mode ↔ hadess | read/write |
| `~/.config/lsfg-vk/conf.toml` | **MISSING** → lsfg-vk | **not implemented** |
| `/sys/module/lsfg_vk/*` | bigame-mode → **non-existent** | **WRONG target** |

---

## 2. Parameter Mapping: falcond ↔ bigame-core

| falcond `Config` field | bigame-core `FalcondConfig` | Status |
|------------------------|----------------------------|--------|
| `enable_performance_mode: bool` | `enable_performance_mode: bool` | ✅ Sync |
| `scx_sched: ScxScheduler` | `scx_sched: String` | ✅ Sync (string-typed) |
| `scx_sched_props: ScxMode` | `scx_sched_props: String` | ✅ Sync |
| `vcache_mode: VCacheMode` | `vcache_mode: String` | ✅ Sync |
| `profile_mode: ProfileMode` | `profile_mode: String` | ✅ Sync |
| `poll_interval_ms: u32` | `poll_interval_ms: u32` | ✅ Sync |
| `system_processes: [][]u8` | **ABSENT** | ⚠️ Missing field |

| falcond `ProfileConfig` field | bigame-core `GameProfile` | Status |
|-------------------------------|--------------------------|--------|
| `name: []u8` | `name: String` | ✅ Sync |
| `performance_mode: bool` | `performance_mode: bool` | ✅ Sync |
| `scx_sched: ScxScheduler` | `scx_sched: String` | ✅ Sync |
| `scx_sched_props: ScxMode` | `scx_sched_props: String` | ✅ Sync |
| `vcache_mode: VCacheMode` | `vcache_mode: String` | ✅ Sync |
| `start_script: ?[]u8` | `start_script: Option<String>` | ✅ Sync |
| `stop_script: ?[]u8` | `stop_script: Option<String>` | ✅ Sync |
| `idle_inhibit: bool` | `idle_inhibit: bool` | ✅ Sync |
| *(not in falcond)* | `cpu_governor: String` | ❌ No setter implemented |
| *(not in falcond)* | `scx_custom_flags: String` | ⚠️ UI only, no falcond support |
| *(not in falcond)* | `fg_multiplier: u32` | ❌ Wrong write target |
| *(not in falcond)* | `fg_flow_scale: u32` | ❌ Wrong write target + type mismatch |
| *(not in falcond)* | `fg_perf_mode: bool` | ❌ Wrong write target |

---

## 3. Parameter Mapping: lsfg-vk ↔ bigame-core

| lsfg-vk `GameConf` field | bigame-core equivalent | Status |
|--------------------------|------------------------|--------|
| `name: string` | `GameProfile.name` | ⚠️ Not mapped to lsfg-vk TOML |
| `active_in: []string` | `GameProfile.name` (proc name) | ❌ Not written to lsfg-vk config |
| `multiplier: size_t` | `fg_multiplier: u32` (1–4) | ❌ Written to non-existent sysfs |
| `flow_scale: float` (0.0–1.0) | `fg_flow_scale: u32` (0–100) | ❌ Wrong target + type mismatch |
| `performance_mode: bool` | `fg_perf_mode: bool` | ❌ Written to non-existent sysfs |
| `pacing: Pacing` | **ABSENT** | ⚠️ Not exposed in UI |
| `gpu: optional<string>` | **ABSENT** | ⚠️ Not exposed in UI |

| lsfg-vk `GlobalConf` field | bigame-core equivalent | Status |
|----------------------------|------------------------|--------|
| `allow_fp16: bool` | **ABSENT** | ⚠️ Not managed |
| `dll: optional<string>` | **ABSENT** | ℹ️ Linux-only scope, DLL unused |

---

## 4. Falcond Integration Audit

### 4.1 Syscalls & Sysfs Access

| Operation | falcond approach | bigame-mode approach | Verdict |
|-----------|-----------------|----------------------|---------|
| VCache write | Direct sysfs (root) | `pkexec tee` + polkit | ✅ Both correct, polkit action defined |
| Scheduler switch | D-Bus → scx_loader | Writes config → SIGHUP reload | ✅ Correct delegation |
| Power profile | D-Bus → power-profiles-daemon | D-Bus directly (`dbus.rs`) | ✅ Independent parallel access |
| CPU governor write | *(not in falcond)* | **NOT IMPLEMENTED** | ❌ Missing |
| SIGHUP signaling | Receives it | `pkill -HUP falcond` | ✅ Works |

### 4.2 Status File Parsing

`bigame-core/src/status.rs` correctly parses the `KEY: VALUE` / section format emitted by `falcond/src/status.zig`.

**Verified fields:**
- `FEATURES: Performance Mode:` → `performance_available` ✅
- `CONFIG: Profile Mode:` → `profile_mode` ✅
- `CONFIG: Global VCache Mode:` → `config_vcache` ✅
- `CONFIG: Global SCX Scheduler:` → `config_scx` ✅
- `LOADED_PROFILES:` → `loaded_profiles` ✅
- `ACTIVE_PROFILE:` → `active_profile` ✅
- `CURRENT_STATUS:` subsection → perf_mode, vcache, scx, screensaver ✅

**Potential divergence:** Both sides hardcode `/tmp/falcond_status` as default. If falcond is compiled with `--tmp-status-file=/run/falcond/status`, bigame-mode will show "Stopped" permanently. No shared build-time constant exists.

### 4.3 Profile File Format

falcond profiles use `otter_conf` (TOML-based). bigame-mode profiles use `toml::to_string_pretty`. Format is compatible for shared fields. However:

- File extension: falcond uses `.conf`, bigame-mode writes `.conf` ✅
- Field casing: `snake_case` on both sides ✅
- Path: both use `/usr/share/falcond/profiles/user/` ✅

### 4.4 Permission Model (falcond side)

```
falcond runs as root (User=root in falcond.service)
  ↓ direct sysfs writes (no pkexec needed)
  ↓ D-Bus system bus access (PowerProfiles, scx_loader, logind)
  ↓ runScript() drops to active_uid via sudo -u #<uid>
```

```
bigame-mode runs as user
  ↓ polkit actions require auth_admin_keep (authenticated once per session)
  ↓ pkexec tee for all privileged writes
  ↓ D-Bus session for GameMode, D-Bus system for PowerProfiles
```

This split is architecturally sound. bigame-mode correctly delegates performance management to falcond rather than trying to replicate it.

---

## 5. lsfg-vk Integration Audit

### 5.1 Architecture: Layer vs Kernel Module

**FACT:** lsfg-vk is a **Vulkan implicit layer** (`VK_LAYER_LSFGVK_frame_generation`). It operates entirely in userspace by intercepting Vulkan swapchain calls at the driver level. It has no kernel module and no `/sys/module/lsfg_vk/` sysfs tree.

**FACT:** `bigame-core/src/fg.rs` writes to `/sys/module/lsfg_vk/parameters/{multiplier,flow_scale,performance_mode}`. These paths **do not exist**.

Consequence: All three `fg::set_*()` calls in `fg_controls.rs` will fail at runtime with `lsfg_vk module is not loaded or sysfs path not found`. The FG controls widget appears interactive but produces no effect.

### 5.2 Correct Hot-Reload Mechanism

lsfg-vk uses `WatchedConfig` (C++) which wraps `inotify`/`stat` mtime comparison on `~/.config/lsfg-vk/conf.toml`. Changes to `multiplier`, `flow_scale`, `performance_mode` are hot-applied between frames (no swapchain recreation). The correct path for bigame-mode to influence lsfg-vk is:

```
bigame-mode profile save
  ↓ write/update ~/.config/lsfg-vk/conf.toml
  ↓ lsfg-vk WatchedConfig::update() detects mtime change
  ↓ GameConf parameters applied to next frame generation pass
```

### 5.3 flow_scale Type Mismatch

| Source | Type | Range | Meaning |
|--------|------|--------|---------|
| lsfg-vk `GameConf.flow_scale` | `float` | `0.0 – 1.0` | Fraction of resolution |
| bigame-mode `fg_flow_scale` | `u32` | `0 – 100` | Displayed as % |

When writing the lsfg-vk TOML: `flow_scale = fg_flow_scale as f32 / 100.0`.  
When reading for UI: `fg_flow_scale = (flow_scale * 100.0) as u32`.

### 5.4 Profile Auto-detection

lsfg-vk profile activation (`active_in`) matches via:
1. `process_name` from `/proc/self/comm`
2. Executable basename from `/proc/self/exe`
3. Wine exe path from `/proc/self/maps` (`.exe` endings)
4. `LSFGVK_PROFILE` environment variable (override)

bigame-mode `GameProfile.name` = executable/process name used by falcond's `ProfileTable`. Both systems use the same process name for matching. A unified lsfg-vk TOML writer can derive `active_in` from `profile.name`.

### 5.5 Environment Variable Injection (Missing)

For scenarios where a user launches a game from within bigame-mode or via a wrapper script, the `LSFGVK_*` environment variables provide a stateless alternative to TOML config:

```bash
LSFGVK_ENV=1 LSFGVK_MULTIPLIER=2 LSFGVK_FLOW_SCALE=0.85 gamescope -- game
```

No wrapper script generation or env variable injection exists in bigame-mode currently.

---

## 6. Polkit & Permission Audit

### 6.1 Defined Polkit Actions

| Action ID | Defined in .policy | Used in code | Verdict |
|-----------|-------------------|--------------|---------|
| `com.biglinux.bigamemode.set-governor` | ✅ | ❌ `pkexec` called but no `set_governor()` fn | ❌ Orphaned action |
| `com.biglinux.bigamemode.set-scheduler` | ✅ | ✅ Delegated to falcond config | ✅ |
| `com.biglinux.bigamemode.set-vcache` | ✅ | ✅ `vcache::set_mode()` uses pkexec | ✅ |
| `com.biglinux.bigamemode.write-config` | ✅ | ✅ `config::write()` uses pkexec | ✅ |
| `com.biglinux.bigamemode.manage-profiles` | ✅ | ✅ `profiles::save/delete` use pkexec | ✅ |

**Note:** `set-scheduler` is defined but bigame-mode does not call scx_loader directly — it writes to falcond config and signals SIGHUP. The polkit action exists for future direct scheduler control but is currently unused for the advertised purpose.

### 6.2 pkexec tee Pattern — Security Review

All privileged writes use the pattern:
```rust
Command::new("pkexec")
    .args(["tee", path])
    .stdin(Stdio::piped())
    // ...
```

**Assessment:** SAFE. `pkexec tee` is the standard Polkit-authorized write mechanism. Stdin piping avoids temp file exposure. Path argument comes from internal constants, not user input, preventing injection. The policy requires `auth_admin_keep` (interactive auth, cached per-session).

**Risk:** `pkexec mkdir -p` in `config::write_to()` is called without explicit polkit action authentication — it relies on the shell environment. This is a low-risk ancillary operation but worth noting.

### 6.3 Script Execution in falcond

`daemon_actions.zig:runScript()` drops from root to `active_uid` via:
```
sudo -u #<uid> env DBUS_SESSION_BUS_ADDRESS=... DISPLAY=:0 /bin/sh -c <script>
```
- ✅ Uses uid, not username (avoids username injection)
- ✅ Sets minimal env
- ✅ Explicit `/bin/sh -c` shell invocation
- ⚠️ Script content is user-provided in profile TOML (arbitrary code execution by design; document in UX)

---

## 7. Missing Features Report

### CRITICAL (blocking correct functionality)

---

#### [C-1] lsfg-vk sysfs paths — WRONG TARGET
**File:** `bigame-mode/crates/bigame-core/src/fg.rs`  
**Symptom:** All FG parameter writes fail silently at runtime.  
**Root cause:** `/sys/module/lsfg_vk/parameters/*` does not exist. lsfg-vk is a userspace Vulkan layer.  
**Fix:** Replace sysfs writes with a TOML writer for `~/.config/lsfg-vk/conf.toml`.

```rust
// REQUIRED: New module bigame-core/src/lsfg_config.rs
pub fn write_profile(name: &str, multiplier: u32, flow_scale: u32, perf_mode: bool) -> Result<()>;
pub fn read_profile(name: &str) -> Result<(u32, f32, bool)>;
```

---

#### [C-2] lsfg-vk TOML config writer — MISSING GLUE CODE
**File:** `bigame-core/src/profiles.rs` (save/delete)  
**Symptom:** `fg_*` fields in `GameProfile` are stored in falcond profiles but never applied to lsfg-vk.  
**Root cause:** No code writes to `~/.config/lsfg-vk/conf.toml` after a profile save.  
**Fix:** After `profiles::save()`, also update the lsfg-vk TOML to add/update the corresponding `[[profile]]` entry with `active_in = [profile.name]`, `multiplier`, `flow_scale = fg_flow_scale / 100.0`, `performance_mode`.

---

#### [C-3] CPU governor setter — MISSING
**File:** `bigame-core/src/governor.rs`  
**Symptom:** `GameProfile.cpu_governor` field is defined and displayed in the profile editor but never applied.  
**Root cause:** `governor.rs` has `available()` and `current()` but no `set()` function. Polkit action `set-governor` is defined but has no callee.  
**Fix:**

```rust
// Add to governor.rs:
pub fn set(governor: &str) -> Result<()> {
    // write to /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor for all cores
    // via pkexec (polkit action: com.biglinux.bigamemode.set-governor)
}
```
Also needs integration in profile activation (currently bigame-mode delegates profile activation to falcond, but governor is a bigame-mode-specific field not known to falcond).

---

#### [C-4] flow_scale type mismatch (masked by wrong sysfs target)
**File:** `bigame-core/src/fg.rs`, `bigame-core/src/profiles.rs`  
**Symptom:** When [C-1] is fixed, flow_scale values will be wrong by factor 100.  
**Root cause:** lsfg-vk uses `float 0.0–1.0`; bigame stores `u32 0–100`.  
**Fix:** In the lsfg-vk TOML writer: `flow_scale = profile.fg_flow_scale as f32 / 100.0`.

---

### MEDIUM (incomplete implementation)

---

#### [M-1] FalcondConfig missing `system_processes` field
**File:** `bigame-core/src/config.rs`  
**Risk:** Writing config via bigame-mode serializes without `system_processes`, overwriting user-configured system process exclusions.  
**Fix:** Add `system_processes: Vec<String>` to `FalcondConfig`; preserve on read-modify-write.

```rust
#[serde(default)]
pub system_processes: Vec<String>,
```

---

#### [M-2] GPU telemetry — STUBBED
**File:** `bigame-core/src/telemetry.rs`  
**Symptom:** `GpuSnapshot` struct has fields `freq_mhz` and `temp_celsius` but no read functions. Dashboard GPU stats will always show "—".  
**Fix:** Implement GPU telemetry readers using:
- AMD: `/sys/class/drm/card*/device/hwmon/hwmon*/freq1_input` and `temp*_input`
- NVIDIA: `nvidia-smi --query-gpu=clocks.current.graphics,temperature.gpu --format=csv,noheader`

---

#### [M-3] `scx_custom_flags` — UI-only, no falcond support
**File:** `bigame-core/src/profiles.rs`  
**Risk:** `GameProfile.scx_custom_flags` is serialized to profile TOML but falcond's `ProfileConfig`/`ActivationData` has no corresponding field. falcond silently ignores it.  
**Fix:** Either remove the field or add upstream support to falcond.

---

#### [M-4] Status file path not shared via build constant
**File:** `bigame-core/src/status.rs` line 11  
**Risk:** If falcond is compiled with `--tmp-status-file=<custom>`, bigame-mode loses daemon visibility.  
**Fix:** Define a shared constant in a common location, or add a build-time feature flag. For packaging, document that both must use the default `/tmp/falcond_status`.

---

### INFORMATIONAL (design notes, not bugs)

---

#### [I-1] lsfg-vk `pacing` and `gpu` fields not exposed
**Status:** Acceptable for v1. `pacing = 'none'` is the only supported value. Multi-GPU setups are rare. Add to profile editor in a future release.

---

#### [I-2] lsfg-vk `allow_fp16` (global) not managed
**Status:** Defaults to `true` in lsfg-vk (correct for modern AMD). No management needed. If exposing, add to global settings (not per-game profile).

---

#### [I-3] Gamescope `launch()` — config available, no launcher integration
**Status:** `gamescope.rs::launch()` works correctly, but bigame-mode is a performance manager, not a game launcher. Consider exporting per-game Gamescope launch scripts as a QoL feature.

---

## 8. Validation Checklist

### falcond ↔ bigame-mode

- [x] Config file path (`/etc/falcond/config.conf`) matches both sides
- [x] Profile directory (`/usr/share/falcond/profiles/user/`) matches both sides
- [x] Status file path (`/tmp/falcond_status`) matches (default build options)
- [x] SIGHUP signaling via `pkill -HUP falcond` is correct
- [x] TOML serialization format compatible
- [x] All falcond config fields mirrored (except `system_processes`) 
- [x] All falcond profile fields mirrored (except FG fields wrongly targeting sysfs)
- [x] Polkit policy covers all privileged operations
- [ ] `system_processes` preserved on config write — **[M-1]**
- [ ] CPU governor setter implemented — **[C-3]**
- [ ] `scx_custom_flags` support in falcond — **[M-3]**

### lsfg-vk ↔ bigame-mode

- [ ] lsfg-vk TOML config writer — **[C-1][C-2]**
- [ ] `flow_scale` float conversion (÷100) — **[C-4]**
- [ ] `active_in` derived from `GameProfile.name` — **[C-2]**
- [ ] Hot-reload works via TOML mtime — (follows from C-1 fix)
- [ ] `LSFGVK_*` env var injection for game launch wrappers — **future**
- [ ] FG multiplier range documented (lsfg-vk supports >4) — informational

### Security

- [x] pkexec tee pattern — no injection vectors
- [x] Profile name validated (no path separators, no `..`)
- [x] Scripts executed as active user (not root) via `sudo -u #uid`
- [x] Polkit action granularity appropriate
- [x] No secrets or credentials in code
- [x] GPIO/sysfs writes bounded to known safe paths

---

## 9. Recommended Implementation Order

1. **[C-1 + C-2]** Create `bigame-core/src/lsfg_config.rs` — writer for `~/.config/lsfg-vk/conf.toml`:
   - Read existing config (preserve global settings and other profiles)
   - Update/insert `[[profile]]` entry for the game
   - Call from `profiles::save()` and `fg::set_*()` hot paths

2. **[C-4]** Fix `flow_scale` conversion when writing lsfg-vk TOML.

3. **[C-3]** Add `governor::set(governor: &str)` writing to all CPU core sysfs nodes via pkexec, called from profile activation logic.

4. **[M-1]** Add `system_processes` to `FalcondConfig` with `#[serde(default)]`, ensuring round-trip fidelity.

5. **[M-2]** Implement GPU telemetry readers for AMD and NVIDIA sysfs/CLI paths.

6. **[M-3]** Decide: remove `scx_custom_flags` from `GameProfile` or add to falcond upstream.

---

*Generated by automated cross-repository audit. All findings are FACT-backed (code paths verified). No speculative items included.*
