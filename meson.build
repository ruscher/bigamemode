project(
  'bigame-mode',
  'rust',
  version: '0.1.0',
  license: 'GPL-3.0-or-later',
  meson_version: '>= 0.63',
)

app_id = 'com.biglinux.BiGameMode'
bin_name = 'bigame-mode'

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
cargo_env = environment()
cargo_env.set('CARGO_HOME', meson.project_build_root() / 'cargo-home')

if get_option('buildtype') == 'release'
  cargo_profile = 'release'
  cargo_args = ['--release']
else
  cargo_profile = 'debug'
  cargo_args = []
endif

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
    cargo, 'build',
    cargo_args,
    '--manifest-path', meson.project_source_root() / 'Cargo.toml',
    '--target-dir', meson.project_build_root() / 'target',
    '&&',
    'cp', meson.project_build_root() / 'target' / cargo_profile / bin_name,
    '@OUTPUT@',
  ],
)

subdir('data')
subdir('po')
