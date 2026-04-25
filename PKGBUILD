# Maintainer: BigLinux Team <dev@biglinux.com.br>

pkgname=bigame-mode
pkgdesc="Gaming performance orchestration for BigLinux (Libadwaita UI + falcond backend)"
pkgver=1.0.0
pkgrel=1
arch=('x86_64')
url="https://github.com/ruscher/bigamemode"
license=('GPL-3.0-or-later')
depends=(
    'gtk4'
    'libadwaita'
    'glib2'
    'falcond'
    'lsfg-vk'
)
makedepends=(
    'rust'
    'cargo'
    'gettext'
)
optdepends=(
    'scx-scheds: Sched-ext schedulers for gaming performance'
    'gamemode: GameMode D-Bus service'
    'power-profiles-daemon: PowerProfiles D-Bus backend'
    'gamescope: micro-compositor with FSR/framerate control'
    'mangohud: in-game performance overlay'
)
source=("${pkgname}::git+${url}.git")
md5sums=('SKIP')

prepare() {
    cd "${srcdir}/${pkgname}"
    export CARGO_HOME="${srcdir}/cargo-home"
    export RUSTFLAGS="${RUSTFLAGS:+${RUSTFLAGS} }--remap-path-prefix=${srcdir}=."
    cargo fetch --locked --manifest-path bigame-engine/Cargo.toml
}

build() {
    cd "${srcdir}/${pkgname}"
    export CARGO_HOME="${srcdir}/cargo-home"
    export RUSTFLAGS="${RUSTFLAGS:+${RUSTFLAGS} }--remap-path-prefix=${srcdir}=."
    # Build the full workspace (bigame-ui and bigame-daemon)
    cargo build --release --locked --manifest-path bigame-engine/Cargo.toml

    # Regenerate .pot template from Rust source (used by translators).
    xgettext \
        --from-code=UTF-8 \
        --keyword=i18n \
        --keyword=gettext \
        --language=Rust \
        --add-comments \
        --output=locale/bigame-mode.pot \
        --package-name="${pkgname}" \
        bigame-engine/bigame-ui/src/views/dashboard.rs \
        bigame-engine/bigame-ui/src/views/logs.rs \
        bigame-engine/bigame-ui/src/views/profile_wizard.rs \
        bigame-engine/bigame-ui/src/views/profiles.rs \
        bigame-engine/bigame-ui/src/views/settings.rs \
        bigame-engine/bigame-ui/src/views/tuning.rs \
        bigame-engine/bigame-ui/src/widgets/booster_toggle.rs \
        bigame-engine/bigame-ui/src/widgets/error_indicator.rs \
        bigame-engine/bigame-ui/src/widgets/fg_controls.rs \
        bigame-engine/bigame-ui/src/widgets/scheduler_info.rs \
        bigame-engine/bigame-ui/src/widgets/tutorial.rs \
        bigame-engine/bigame-ui/src/tray.rs \
        bigame-engine/bigame-ui/src/window.rs \
        bigame-engine/bigame-ui/src/app.rs

    # Merge new template strings into existing .po files (preserves translations).
    for po in locale/*.po; do
        msgmerge --no-fuzzy-matching -q -U "${po}" locale/bigame-mode.pot
    done

    # Compile .po -> .mo binaries.
    for po in locale/*.po; do
        lang=$(basename "${po}" .po)
        mkdir -p "locale/${lang}/LC_MESSAGES"
        msgfmt "${po}" -o "locale/${lang}/LC_MESSAGES/${pkgname}.mo"
    done
}

check() {
    cd "${srcdir}/${pkgname}"
    export CARGO_HOME="${srcdir}/cargo-home"
    export RUSTFLAGS="${RUSTFLAGS:+${RUSTFLAGS} }--remap-path-prefix=${srcdir}=."
    cargo test --release --locked --manifest-path bigame-engine/Cargo.toml
}

package() {
    cd "${srcdir}/${pkgname}"

    # bigame-ui binary / matches desktop launcher Exec key
    install -Dm755 "bigame-engine/target/release/bigame-ui" \
        "${pkgdir}/usr/bin/bigame-ui"

    # bigame-daemon binary
    install -Dm755 "bigame-engine/target/release/bigame-daemon" \
        "${pkgdir}/usr/bin/bigame-daemon"

    # Diagnostic script
    install -Dm755 "usr/bin/falcond-diag" \
        "${pkgdir}/usr/bin/falcond-diag"

    # Desktop file
    install -Dm644 "data/com.biglinux.BiGameMode.desktop" \
        "${pkgdir}/usr/share/applications/com.biglinux.BiGameMode.desktop"

    # Metainfo
    install -Dm644 "data/com.biglinux.BiGameMode.metainfo.xml" \
        "${pkgdir}/usr/share/metainfo/com.biglinux.BiGameMode.metainfo.xml"

    # Polkit policy
    install -Dm644 "data/com.biglinux.BiGameMode.policy" \
        "${pkgdir}/usr/share/polkit-1/actions/com.biglinux.BiGameMode.policy"

    # D-Bus policies
    install -Dm644 "data/com.biglinux.BiGameMode.conf" \
        "${pkgdir}/usr/share/dbus-1/system.d/com.biglinux.BiGameMode.conf"

    # Systemd services
    install -Dm644 "data/bigame-daemon.service" \
        "${pkgdir}/usr/lib/systemd/system/bigame-daemon.service"

    # D-Bus activation service
    install -Dm644 "data/com.biglinux.BiGameMode.service" \
        "${pkgdir}/usr/share/dbus-1/system-services/com.biglinux.BiGameMode.service"

    # Icon
    install -Dm644 "data/icons/com.biglinux.BiGameMode.svg" \
        "${pkgdir}/usr/share/icons/hicolor/scalable/apps/com.biglinux.BiGameMode.svg"

    # License
    install -Dm644 LICENSE "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE"
}
