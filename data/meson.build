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

# Application icon (scalable SVG)
install_data(
  'icons' / app_id + '.svg',
  install_dir: iconsdir / 'scalable' / 'apps',
)
