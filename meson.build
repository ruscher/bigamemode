project(
  'bigame-mode',
  'rust',
  version: '1.0.0',
  license: 'GPL-3.0-or-later',
  meson_version: '>= 0.63',
)

app_id = 'com.biglinux.BiGameMode'
bin_name = 'bigame-ui'

# Directories
prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
datadir = prefix / get_option('datadir')
localedir = prefix / get_option('localedir')
iconsdir = datadir / 'icons' / 'hicolor'

# Dependencies
dependency('gtk4', version: '>= 4.14')
dependency('libadwaita-1', version: '>= 1.6')

# Cargo build
cargo = find_program('cargo')
sh = find_program('sh')
cargo_env = environment()
cargo_env.set('CARGO_HOME', meson.project_build_root() / 'cargo-home')

cargo_manifest = meson.project_source_root() / 'bigame-engine' / 'Cargo.toml'
cargo_target_dir = meson.project_build_root() / 'target'

if get_option('buildtype') == 'release'
  cargo_profile = 'release'
  cargo_build_flags = '--release'
else
  cargo_profile = 'debug'
  cargo_build_flags = ''
endif

cargo_output = cargo_target_dir / cargo_profile / bin_name

cargo_build = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: bin_name,
  console: true,
  install: true,
  install_dir: bindir,
  env: cargo_env,
  command: [
    sh,
    '-c',
    'set -e; cargo build ' + cargo_build_flags + ' --manifest-path "$1" --target-dir "$2"; cp "$3" "$4"',
    'meson-cargo-build',
    cargo_manifest,
    cargo_target_dir,
    cargo_output,
    '@OUTPUT@',
  ],
)

subdir('data')
subdir('locale')
