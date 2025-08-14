#!/bin/sh

set -xe

PROJECT_ROOT="$(dirname $(realpath $0))"
DEPS="$PROJECT_ROOT/3rd_party"
TOOLCHAINS="$PROJECT_ROOT/toolchains"

case "$1" in
    "--linux")
        source "$TOOLCHAINS/build-linux.sh"
    ;;
    "--mingw")
        source "$TOOLCHAINS/build-mingw.sh"
    ;;
    "--mingw64")
        source "$TOOLCHAINS/build-mingw64.sh"
    ;;
    *)
        echo "This build script accepts the following arguments"
        echo "--linux   -   To build with the local linux toolchain"
        echo "--mingw   -   To build with the i686 mingw toolchain (from mingw64 nowadays)"
        echo "--mingw64 -   To build with the x86_64 mingw toolchain (from mingw64 nowadays)"
        exit 1
    ;;
esac


cmake_build "$DEPS/zlib"

cmake_build "$DEPS/brotli"

cmake_build "$DEPS/zstd/build/cmake"

cmake_build "$DEPS/json" \
    -DJSON_BuildTests=OFF \
    -DJSON_Install=ON \
    -DJSON_MultipleHeaders=OFF

cmake_build "$DEPS/SDL"

config_ssl

# Only one job even if it's a makefile, openssl's config of it is broken
# if multiple jobs are used
set +e
make -C "$BUILD/openssl" -j $(( $(nproc) * 2 )) install
set -e
make -C "$BUILD/openssl" install

# cmake_build "$DEPS/nghttp3" \
#     -DENABLE_STATIC_LIBS=ON \
#     -DENABLE_SHARED_LIBS=OFF \
#     -DBUILD_TESTING=OFF
#
# cmake_build "$DEPS/ngtcp2" \
#     -DENABLE_STATIC_LIBS=ON \
#     -DENABLE_SHARED_LIBS=OFF \
#     -DENABLE_OPENSSL=ON \
#     -DBUILD_TESTING=OFF
#
# cmake_build "$DEPS/nghttp2" \
#     -DENABLE_STATIC_LIBS=ON \
#     -DENABLE_SHARED_LIBS=OFF \
#     -DENABLE_LIB_ONLY=ON \
#     -DENABLE_DOC=OFF \
#     -DBUILD_TESTING=OFF
#
# cmake_build "$DEPS/curl" \
#     -DBUILD_CURL_EXE=OFF \
#     -DBUILD_EXAMPLES=OFF \
#     -DBUILD_LIBCURL_DOCS=OFF \
#     -DBUILD_MISC_DOCS=OFF \
#     -DBUILD_TESTING=OFF \
#     -DCURL_DEFAULT_SSL_BACKEND=openssl \
#     -DCURL_LTO=ON \
#     -DCURL_CA_FALLBACK=ON \
#     -DCURL_ENABLE_SSL=ON \
#     -DCURL_USE_OPENSSL=ON \
#     -DCURL_USE_NGHTTP2=ON \
#     -DCURL_USE_NGTCP2=ON \
#     -DOPENSSL_ROOT_DIR="$PREFIX"

cmake_build "$DEPS/cpp-httplib"

build_launcher "$PROJECT_ROOT"
