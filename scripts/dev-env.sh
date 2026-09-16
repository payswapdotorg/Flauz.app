#!/bin/sh
# Userspace build environment for the Flauz.app workspace on machines
# without root system-package access (GUI-001 development sandbox).
#
# The sysroot prefix is materialized from stock Debian .debs under
# /home/z/sysroot; nothing here modifies the host system.
export PATH="$HOME/.cargo/bin:$PATH"
export PKG_CONFIG_SYSROOT_DIR=/home/z/sysroot/prefix
export PKG_CONFIG_PATH="/home/z/sysroot/prefix/usr/lib/x86_64-linux-gnu/pkgconfig:/home/z/sysroot/prefix/usr/share/pkgconfig"
export LIBCLANG_PATH="/home/z/sysroot/prefix/usr/lib/llvm-19/lib"
export LD_LIBRARY_PATH="/home/z/sysroot/prefix/usr/lib/x86_64-linux-gnu:/home/z/sysroot/prefix/usr/lib/llvm-19/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export BINDGEN_EXTRA_CLANG_ARGS="-I/home/z/sysroot/prefix/usr/lib/llvm-19/lib/clang/19/include"
export CPATH="/home/z/sysroot/prefix/usr/include:/home/z/sysroot/prefix/usr/include/pipewire-0.3:/home/z/sysroot/prefix/usr/include/spa-0.2"
export CARGO_INCREMENTAL=0
