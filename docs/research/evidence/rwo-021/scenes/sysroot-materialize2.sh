#!/usr/bin/env bash
# RWO-021: sysroot materializer v2 — explicit list + one-level deps (fast, lean)
set -u
ROOT=/home/z/sysroot
PREFIX=$ROOT/prefix
DEBS=$ROOT/debs
mkdir -p "$DEBS" "$PREFIX"
cd "$DEBS" || exit 1

TOP="libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libpipewire-0.3-dev \
libegl1-mesa-dev libgbm-dev libdrm-dev libx11-xcb-dev libxcb-render0-dev \
libxcb-shape0-dev libxcb-xfixes0-dev libclang-19-dev libclang-cpp19 \
mesa-vulkan-drivers libllvm19 xdotool x11-utils picom \
libxkbcommon0 libxkbcommon-x11-0 libwayland-client0 libwayland-cursor0 \
libwayland-server0 libegl1 libegl-mesa0 libgl1 libglvnd0 libglx0 libopengl0 \
libgbm1 libdrm2 libpipewire-0.3-0 libspa-0.2-0 libspa-0.2-dev libxcb-cursor0"

echo "== [1] one-level dependency expansion =="
LIST="$TOP"
for p in $TOP; do
  deps=$(apt-cache depends --no-recommends --no-suggests --no-conflicts \
    --no-breaks --no-replaces --no-enhances "$p" 2>/dev/null \
    | awk '/^ *[A-Z][a-z]+: /{print}' | grep -E "^ *Depends: " \
    | sed 's/^ *Depends: //; s/ *//g' | grep -vE "^<" | sort -u)
  LIST="$LIST $deps"
done
# dedupe
LIST=$(echo "$LIST" | tr ' ' '\n' | sort -u | grep -v '^$')
echo "$LIST" | wc -l | xargs echo "packages to fetch:"

echo "== [2] download =="
: > failed.txt
for p in $LIST; do
  if ! ls "${p}"_*.deb >/dev/null 2>&1 && ! ls "${p}"_*.deb >/dev/null 2>&1; then
    apt-get download "$p" >/dev/null 2>&1 || echo "$p" >> failed.txt
  fi
done
echo "downloaded: $(ls ./*.deb 2>/dev/null | wc -l); failed: $(sort -u failed.txt | grep -c . || true)"
sort -u failed.txt | head -20

echo "== [3] extract =="
n=0
for d in ./*.deb; do
  dpkg -x "$d" "$PREFIX" && n=$((n+1))
done
echo "extracted: $n debs into $PREFIX"

echo "== [4] verify =="
export PKG_CONFIG_SYSROOT_DIR=$PREFIX
export PKG_CONFIG_PATH="$PREFIX/usr/lib/x86_64-linux-gnu/pkgconfig:$PREFIX/usr/share/pkgconfig"
for p in xkbcommon xkbcommon-x11 x11 xcb x11-xcb xcb-render xcb-shape xcb-xfixes wayland-client wayland-cursor wayland-server fontconfig freetype2 libpipewire-0.3 egl gbm drm gl; do
  printf "%-18s " "$p"
  pkg-config --modversion "$p" 2>/dev/null || echo MISSING
done
export LD_LIBRARY_PATH="$PREFIX/usr/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"
echo "--- tool probes ---"
"$PREFIX/usr/bin/picom" --version 2>&1 | head -1 || echo "picom MISSING"
"$PREFIX/usr/bin/xdotool" version 2>&1 | head -1 || echo "xdotool MISSING"
"$PREFIX/usr/bin/xwininfo" -version 2>&1 | head -1 || echo "xwininfo MISSING"
ls "$PREFIX/usr/share/vulkan/icd.d/" 2>/dev/null || echo "no vulkan ICD dir"
ls "$PREFIX/usr/lib/llvm-19/lib/" 2>/dev/null | grep -c libclang
echo "SYSROOT_V2_DONE"
