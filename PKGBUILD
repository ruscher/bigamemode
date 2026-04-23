# Maintainer: BigLinux Team <dev@biglinux.com.br>

pkgname=bigame-mode
pkgdesc="Gaming performance orchestration for BigLinux (Libadwaita UI + falcond backend)"
pkgver=0.1.0
pkgrel=1
arch=('x86_64')
url="https://github.com/ruscher/bigamemode"
license=('GPL-3.0-or-later')
depends=(
    'gtk4'
    'libadwaita'
    'dbus'
    'polkit'
    'gettext'
)
makedepends=(
    'rust'
    'cargo'
    'gettext'
)
optdepends=(
    'falcond: daemon for automatic game profile switching'
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
    cargo fetch --locked --manifest-path bigame-engine/Cargo.toml
}

build() {
    cd "${srcdir}/${pkgname}"
    export CARGO_HOME="${srcdir}/cargo-home"
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
    cargo test --release --locked --manifest-path bigame-engine/Cargo.toml
}

package() {
    cd "${srcdir}/${pkgname}"

    # bigame-ui binary
    install -Dm755 "bigame-engine/target/release/bigame-ui" \
        "${pkgdir}/usr/bin/bigame-mode"

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
    install -Dm644 "data/org.falcond.conf" \
        "${pkgdir}/usr/share/dbus-1/system.d/org.falcond.conf"

    # Systemd services
    install -Dm644 "data/falcond.service" \
        "${pkgdir}/usr/lib/systemd/system/falcond.service"
    install -Dm644 "data/bigame-daemon.service" \
        "${pkgdir}/usr/lib/systemd/system/bigame-daemon.service"

    # D-Bus activation service
    install -Dm644 "data/com.biglinux.BiGameMode.service" \
        "${pkgdir}/usr/share/dbus-1/system-services/com.biglinux.BiGameMode.service"

    # Sysusers
    install -Dm644 "data/falcond.sysusers" \
        "${pkgdir}/usr/lib/sysusers.d/falcond.conf"

    # Default falcond configuration
    install -Dm644 "data/falcond.conf" \
        "${pkgdir}/etc/falcond/falcond.conf"

    # Icon
    install -Dm644 "data/icons/com.biglinux.BiGameMode.svg" \
        "${pkgdir}/usr/share/icons/hicolor/scalable/apps/com.biglinux.BiGameMode.svg"

    # Translations
    for po in locale/*.po; do
        lang=$(basename "${po}" .po)
        if [ -f "locale/${lang}/LC_MESSAGES/${pkgname}.mo" ]; then
            install -Dm644 "locale/${lang}/LC_MESSAGES/${pkgname}.mo" \
                "${pkgdir}/usr/share/locale/${lang}/LC_MESSAGES/${pkgname}.mo"
        fi
    done

    # License
    install -Dm644 LICENSE "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE"

    # Sudoers: NOPASSWD rules so bigame-mode never prompts for password
    # when writing falcond config, profiles, or VCache sysfs settings.
    install -Dm440 "data/bigame-mode-sudoers" \
        "${pkgdir}/etc/sudoers.d/bigame-mode"
}
