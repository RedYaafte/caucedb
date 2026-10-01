#!/bin/sh
# Install the published Linux x86-64 CauceDB binary. Oracle Instant Client is separate.
set -eu

version=${CAUCEDB_VERSION:-v0.1.0-alpha.2}
install_dir=${CAUCEDB_INSTALL_DIR:-}
tmp_dir=
staged_binary=

fail() {
    printf 'caucedb installer: %s\n' "$1" >&2
    exit 1
}

cleanup() {
    if [ -n "$staged_binary" ]; then
        rm -f "$staged_binary"
    fi
    if [ -n "$tmp_dir" ]; then
        rm -f "$tmp_dir/archive.tar.gz" "$tmp_dir/archive.tar.gz.sha256" "$tmp_dir/caucedb"
        rmdir "$tmp_dir" 2>/dev/null || true
    fi
}

trap cleanup 0
trap 'exit 1' 1 2 3 15

if [ -z "$install_dir" ]; then
    [ -n "${HOME:-}" ] || fail 'HOME is unset; set CAUCEDB_INSTALL_DIR to an absolute path'
    install_dir=$HOME/.local/bin
fi
[ "$(uname -s)" = Linux ] || fail 'only Linux is supported by this release'
[ "$(uname -m)" = x86_64 ] || fail 'only x86-64 is supported by this release'
case "$version" in
    v[0-9]*) ;;
    *) fail 'CAUCEDB_VERSION must be a release tag such as v0.1.0-alpha.1' ;;
esac
case "$version" in
    *[!A-Za-z0-9._-]*) fail 'CAUCEDB_VERSION contains invalid characters' ;;
esac
case "$install_dir" in
    /*) ;;
    *) fail 'CAUCEDB_INSTALL_DIR must be an absolute path' ;;
esac

for required in curl tar sha256sum mktemp install mv awk; do
    command -v "$required" >/dev/null 2>&1 || fail "missing required command: $required"
done

asset="caucedb-${version}-x86_64-unknown-linux-gnu.tar.gz"
base="https://github.com/RedYaafte/caucedb/releases/download/${version}"
tmp_dir=$(mktemp -d "${TMPDIR:-/tmp}/caucedb-install.XXXXXX") || fail 'cannot create temporary directory'

printf 'Downloading CauceDB %s for Linux x86-64...\n' "$version"
curl -fLsS --proto '=https' --proto-redir '=https' --tlsv1.2 --retry 3 --connect-timeout 10 --max-time 120 \
    "$base/$asset" -o "$tmp_dir/archive.tar.gz" || fail 'binary download failed'
curl -fLsS --proto '=https' --proto-redir '=https' --tlsv1.2 --retry 3 --connect-timeout 10 --max-time 30 \
    "$base/$asset.sha256" -o "$tmp_dir/archive.tar.gz.sha256" || fail 'checksum download failed'

# The release checksum names the original asset, so verify it under that name.
expected=$(awk '{print $1}' "$tmp_dir/archive.tar.gz.sha256")
case "$expected" in
    *[!0-9A-Fa-f]*|'') fail 'invalid release checksum' ;;
esac
[ "${#expected}" -eq 64 ] || fail 'invalid release checksum length'
actual=$(sha256sum "$tmp_dir/archive.tar.gz" | awk '{print $1}')
[ "$actual" = "$expected" ] || fail 'SHA-256 verification failed; nothing was installed'
printf 'SHA-256 verified.\n'

tar -xzf "$tmp_dir/archive.tar.gz" -C "$tmp_dir" caucedb || fail 'archive extraction failed'
[ -f "$tmp_dir/caucedb" ] && [ ! -L "$tmp_dir/caucedb" ] || fail 'archive does not contain a regular caucedb binary'
mkdir -p "$install_dir" || fail "cannot create $install_dir"
staged_binary=$(mktemp "$install_dir/.caucedb.XXXXXX") || fail "cannot write to $install_dir"
install -m 755 "$tmp_dir/caucedb" "$staged_binary" || fail 'cannot stage the binary'
mv -f "$staged_binary" "$install_dir/caucedb" || fail 'cannot install the binary'
staged_binary=

printf 'Installed %s\n' "$install_dir/caucedb"
case ":${PATH:-}:" in
    *":$install_dir:"*) printf 'Run: caucedb\n' ;;
    *) printf 'Add it to PATH: export PATH="%s:$PATH"\n' "$install_dir" ;;
esac
printf 'Oracle connections also require Oracle Instant Client installed separately.\n'
