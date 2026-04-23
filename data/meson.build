# Desktop file
install_data(
  app_id + '.desktop',
  install_dir: datadir / 'applications',
)

# Metainfo / AppStream
install_data(
  app_id + '.metainfo.xml',
  install_dir: datadir / 'metainfo',
)

# PolicyKit policy
install_data(
  app_id + '.policy',
  install_dir: datadir / 'polkit-1' / 'actions',
)

# Default configuration for falcond
install_data(
  'falcond.conf',
  install_dir: '/etc/falcond',
)

# D-Bus policy for bigame-daemon
install_data(
  'com.biglinux.BiGameMode.conf',
  install_dir: datadir / 'dbus-1' / 'system.d',
)

# D-Bus policy for falcond
install_data(
  'org.falcond.conf',
  install_dir: datadir / 'dbus-1' / 'system.d',
)

# Sysusers for falcond group
install_data(
  'falcond.sysusers',
  install_dir: prefix / 'lib' / 'sysusers.d',
  rename: 'falcond.conf',
)

# D-Bus service for activation
install_data(
  'com.biglinux.BiGameMode.service',
  install_dir: datadir / 'dbus-1' / 'system-services',
)

# Systemd service for bigame-daemon
install_data(
  'bigame-daemon.service',
  install_dir: prefix / 'lib' / 'systemd' / 'system',
)

# Systemd service for falcond
install_data(
  'falcond.service',
  install_dir: prefix / 'lib' / 'systemd' / 'system',
)

# Application icon (scalable SVG)
install_data(
  'icons' / app_id + '.svg',
  install_dir: iconsdir / 'scalable' / 'apps',
)
