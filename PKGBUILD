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
    'gamemode: GameMode D-Bus service'
    'power-profiles-daemon: PowerProfiles D-Bus backend'
    'gamescope: micro-compositor with FSR/framerate control'
    'mangohud: in-game performance overlay'
)
source=("${pkgname}::git+${url}.git")
md5sums=('SKIP')

prepare() {
    cd "${srcdir}/${pkgname}/bigame-mode"
    export CARGO_HOME="${srcdir}/cargo-home"
    cargo fetch --locked
}

build() {
    cd "${srcdir}/${pkgname}/bigame-mode"
    export CARGO_HOME="${srcdir}/cargo-home"
    cargo build --release --locked

    # Regenerate .pot template from Rust source (used by translators).
    xgettext \
        --from-code=UTF-8 \
        --keyword=i18n \
        --keyword=gettext \
        --language=C \
        --add-comments \
        --output=po/bigame-mode.pot \
        --package-name="${pkgname}" \
        crates/bigame-ui/src/views/dashboard.rs \
        crates/bigame-ui/src/views/gamescope.rs \
        crates/bigame-ui/src/views/logs.rs \
        crates/bigame-ui/src/views/profiles.rs \
        crates/bigame-ui/src/views/settings.rs \
        crates/bigame-ui/src/views/tuning.rs \
        crates/bigame-ui/src/widgets/booster_toggle.rs \
        crates/bigame-ui/src/widgets/fg_controls.rs \
        crates/bigame-ui/src/widgets/tutorial.rs \
        crates/bigame-ui/src/tray.rs \
        crates/bigame-ui/src/window.rs \
        crates/bigame-ui/src/app.rs

    # Merge new template strings into existing .po files (preserves translations).
    for po in po/*.po; do
        msgmerge --no-fuzzy-matching -q -U "${po}" po/bigame-mode.pot
    done

    # Compile .po → .mo binaries.
    for po in po/*.po; do
        lang=$(basename "${po}" .po)
        mkdir -p "po/${lang}/LC_MESSAGES"
        msgfmt "${po}" -o "po/${lang}/LC_MESSAGES/${pkgname}.mo"
    done
}

check() {
    cd "${srcdir}/${pkgname}/bigame-mode"
    export CARGO_HOME="${srcdir}/cargo-home"
    cargo test --release --locked
}

package() {
    cd "${srcdir}/${pkgname}/bigame-mode"

    # Binary
    install -Dm755 "target/release/bigame-ui" \
        "${pkgdir}/usr/bin/${pkgname}"

    # Desktop file
    install -Dm644 "data/com.biglinux.BiGameMode.desktop" \
        "${pkgdir}/usr/share/applications/com.biglinux.BiGameMode.desktop"

    # Metainfo
    install -Dm644 "data/com.biglinux.BiGameMode.metainfo.xml" \
        "${pkgdir}/usr/share/metainfo/com.biglinux.BiGameMode.metainfo.xml"

    # Polkit policy
    install -Dm644 "data/com.biglinux.BiGameMode.policy" \
        "${pkgdir}/usr/share/polkit-1/actions/com.biglinux.BiGameMode.policy"

    # Icon
    install -Dm644 "data/icons/com.biglinux.BiGameMode.svg" \
        "${pkgdir}/usr/share/icons/hicolor/scalable/apps/com.biglinux.BiGameMode.svg"

    # Translations
    for po in po/*.po; do
        lang=$(basename "${po}" .po)
        install -Dm644 "po/${lang}/LC_MESSAGES/${pkgname}.mo" \
            "${pkgdir}/usr/share/locale/${lang}/LC_MESSAGES/${pkgname}.mo"
    done

    # License
    install -Dm644 LICENSE "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE"

    # Sudoers: NOPASSWD rules so bigame-mode never prompts for password
    # when writing falcond config, profiles, or VCache sysfs settings.
    install -Dm440 "data/bigame-mode-sudoers" \
        "${pkgdir}/etc/sudoers.d/bigame-mode"
}
