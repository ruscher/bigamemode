#!/usr/bin/env bash
# What a BiGame-mode working tree holds besides its sources, and a way to
# remove it without touching anything that cannot be rebuilt.
#
#   clean-dev.sh [--analyze]   show sizes, remove nothing (the default)
#   clean-dev.sh --safe        remove what a build recreates: Cargo's target/,
#                              makepkg's src/ and pkg/, built packages, Python
#                              bytecode, editor and msgmerge backups
#   clean-dev.sh --deep [--yes]
#                              --safe, plus: benchmark raw captures moved out
#                              of the tree to $XDG_DATA_HOME/bigame-mode/
#                              benchmarks/raw (only the newest RAW_KEEP
#                              sessions, 20 by default, are kept there).
#                              Lists everything first and asks, unless --yes.
#
# Only files git ignores are candidates: nothing tracked, nothing untracked
# and unknown. A git repository nested in the tree (an old clone) is reported,
# never removed. Nothing outside the tree is touched except the raw-capture
# archive above: not the Steam library, not Wine prefixes, not ~/.cache,
# not the user's Lossless.dll.
set -euo pipefail

die() { echo "clean-dev: $*" >&2; exit 1; }

ROOT=$(git -C "$(dirname "$0")" rev-parse --show-toplevel 2>/dev/null) ||
    die "not inside a git working tree"
[[ -f $ROOT/bigame-engine/Cargo.toml && -f $ROOT/PKGBUILD ]] ||
    die "$ROOT is not a BiGame-mode tree"
cd "$ROOT"

MODE=analyze
YES=0
for a in "$@"; do
    case $a in
        --analyze) MODE=analyze ;;
        --safe) MODE=safe ;;
        --deep) MODE=deep ;;
        --yes) YES=1 ;;
        -h | --help) sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) die "unknown option $a (--analyze, --safe, --deep, --yes)" ;;
    esac
done

ARCHIVE="${XDG_DATA_HOME:-$HOME/.local/share}/bigame-mode/benchmarks/raw"
RAW_KEEP=${RAW_KEEP:-20}
[[ $RAW_KEEP =~ ^[0-9]+$ ]] || die "RAW_KEEP must be a number"

size() { du -sb "$@" 2>/dev/null | awk '{s += $1} END {print s + 0}'; }
human() { numfmt --to=iec --suffix=B "$1"; }
ignored() { git check-ignore -q -- "$1"; }
# Total bytes of the NUL-separated file list on stdin.
total_of() { du -cb --files0-from=- 2>/dev/null | awk 'END {print $1 + 0}'; }

# What --safe removes, each checked to be ignored by git.
safe_targets() {
    local p
    # Cargo marks its own directory; anything else called target/ is left.
    [[ -f bigame-engine/target/CACHEDIR.TAG ]] && echo bigame-engine/target
    for p in src pkg pkgbuild; do
        [[ -d $p ]] && ignored "$p/" && echo "$p"
    done
    git ls-files --others --ignored --exclude-standard --directory -z |
        while IFS= read -r -d '' p; do
            p=${p%/}
            case $p in
                *.pkg.tar | *.pkg.tar.* | *.src.tar.gz) echo "$p" ;;
                */__pycache__ | __pycache__) echo "$p" ;;
                *~ | *.bak | *.orig | *.swp) echo "$p" ;;
            esac
        done
}

# Benchmark raw captures: ignored files under bigame-engine/benchmarks.
raw_captures() {
    git ls-files --others --ignored --exclude-standard -z -- bigame-engine/benchmarks |
        tr '\0' '\n'
}

# Git repositories inside the tree that are not this one (never removed).
nested_repos() {
    find . \( -path ./.git -o -path ./bigame-engine/target \) -prune -o \
        -name .git -printf '%h\n' 2>/dev/null
}

report() {
    local total safe raw repos p
    total=$(size .)
    echo "Tree: $ROOT"
    printf '  %-34s %10s\n' "everything" "$(human "$total")"
    printf '  %-34s %10s\n' "tracked files" "$(human "$(git ls-files -z | total_of)")"
    printf '  %-34s %10s\n' ".git" "$(human "$(size .git)")"
    echo "Removed by --safe:"
    safe=0
    while IFS= read -r p; do
        [[ -n $p ]] || continue
        printf '  %-34s %10s\n' "$p" "$(human "$(size "$p")")"
        safe=$((safe + $(size "$p")))
    done < <(safe_targets)
    printf '  %-34s %10s\n' "(total)" "$(human "$safe")"
    raw=$(raw_captures | wc -l)
    echo "Moved out by --deep: $raw benchmark raw capture(s), $(human "$(raw_captures | tr '\n' '\0' | total_of)") -> $ARCHIVE"
    repos=$(nested_repos)
    if [[ -n $repos ]]; then
        echo "Nested git repositories (not removed; check them and delete by hand):"
        while IFS= read -r p; do
            printf '  %-34s %10s\n' "$p" "$(human "$(size "$p")")"
        done <<<"$repos"
    fi
}

remove_safe() {
    local p
    while IFS= read -r p; do
        [[ -n $p ]] || continue
        if [[ $p == bigame-engine/target ]] && command -v cargo >/dev/null; then
            cargo clean -q --manifest-path bigame-engine/Cargo.toml
        else
            rm -rf -- "$p"
        fi
        echo "removed $p"
    done < <(safe_targets)
}

archive_raw() {
    local f n
    n=0
    while IFS= read -r f; do
        [[ -n $f && -f $f ]] || continue
        mkdir -p "$ARCHIVE/$(dirname "${f#bigame-engine/benchmarks/}")"
        mv -n -- "$f" "$ARCHIVE/${f#bigame-engine/benchmarks/}"
        n=$((n + 1))
    done < <(raw_captures)
    echo "moved $n raw capture(s) to $ARCHIVE"
    # Retention: the newest RAW_KEEP sessions, by the date in their name.
    [[ -d $ARCHIVE ]] || return 0
    find "$ARCHIVE" -mindepth 1 -maxdepth 1 -type d -printf '%f\n' | sort -r |
        tail -n +"$((RAW_KEEP + 1))" | while IFS= read -r f; do
            rm -rf -- "${ARCHIVE:?}/$f"
            echo "removed archived session $f (keeping the newest $RAW_KEEP)"
        done
}

report
case $MODE in
    analyze) ;;
    safe) echo; remove_safe ;;
    deep)
        echo
        echo "--deep will remove the --safe list above and move the raw captures."
        if [[ -d $ARCHIVE ]]; then
            old=$(find "$ARCHIVE" -mindepth 1 -maxdepth 1 -type d -printf '%f\n' |
                sort -r | tail -n +"$((RAW_KEEP + 1))")
            [[ -n $old ]] && printf 'Archived sessions over the %s kept:\n%s\n' "$RAW_KEEP" "$old"
        fi
        if ((YES == 0)); then
            read -r -p "Go on? [y/N] " answer
            [[ $answer == [yY]* ]] || die "nothing removed"
        fi
        remove_safe
        archive_raw
        ;;
esac
