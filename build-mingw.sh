#!/bin/sh

set -xe

TARGET="x86_64-pc-windows-gnu"
PROJECT_ROOT="$(dirname $(realpath $0))"
DEPS="$PROJECT_ROOT/3rd_party"
PREFIX="$PROJECT_ROOT/build/$TARGET"
BUILD="$PREFIX/artifact"

cmake_build()
{
    PROJECT_DIR=$1
    PROJECT=$(basename $PROJECT_DIR)
    ARTIFACT="$BUILD/$PROJECT"

    cmake \
        -B "$ARTIFACT" \
        -DCMAKE_TOOLCHAIN_FILE="$PROJECT_ROOT/toolchains/mingw-64.cmake" \
        -DCMAKE_BUILD_TYPE="Release" \
        -DBUILD_SHARED_LIBS=OFF \
        -DBUILD_STATIC_LIBS=ON \
        -DCMAKE_PREFIX_PATH="$PREFIX" \
        -DCMAKE_INSTALL_PREFIX="$PREFIX" \
        -DCMAKE_MODULE_PATH="$PREFIX/lib/cmake;$PREFIX/lib64/cmake" \
        ${@:3} "$PROJECT_DIR"

    cmake \
        --build "$ARTIFACT" \
        --parallel $(( $(nproc) * 2 )) \
        --target install
}

if ! [ -d "$BUILD/openssl" ]; then
    mkdir --parents "$BUILD/openssl"
    pushd "$BUILD/openssl"
    perl "$DEPS/openssl/Configure" \
        mingw \
        --cross-compile-prefix=x86_64-w64-mingw32- \
        --prefix="$PREFIX" \
        no-shared \
        no-docs \
        no-demos \
        no-h3demo \
        no-tests

    popd
fi

# Only one job as even if it's a makefile, openssl's config of it is broken
# if multiple jobs are used
make -C "$BUILD/openssl" install

cmake_build "$DEPS/json" \
    -DJSON_BuildTests=OFF \
    -DJSON_Install=ON \
    -DJSON_MultipleHeaders=OFF

cmake_build "$DEPS/SDL"

cmake_build "$DEPS/nghttp3" \
    -DENABLE_STATIC_LIBS=ON \
    -DENABLE_SHARED_LIBS=OFF \
    -DBUILD_TESTING=OFF

cmake_build "$DEPS/ngtcp2" \
    -DENABLE_STATIC_LIBS=ON \
    -DENABLE_SHARED_LIBS=OFF \
    -DENABLE_OPENSSL=ON \
    -DBUILD_TESTING=OFF

cmake_build "$DEPS/nghttp2" \
    -DENABLE_STATIC_LIBS=ON \
    -DENABLE_SHARED_LIBS=OFF \
    -DENABLE_LIB_ONLY=ON \
    -DENABLE_DOC=OFF \
    -DBUILD_TESTING=OFF

cmake_build "$DEPS/curl" \
    -DBUILD_CURL_EXE=OFF \
    -DBUILD_EXAMPLES=OFF \
    -DBUILD_LIBCURL_DOCS=OFF \
    -DBUILD_MISC_DOCS=OFF \
    -DBUILD_TESTING=OFF \
    -DCURL_DEFAULT_SSL_BACKEND=openssl \
    -DCURL_LTO=ON \
    -DCURL_CA_FALLBACK=ON \
    -DCURL_ENABLE_SSL=ON \
    -DCURL_USE_OPENSSL=ON \
    -DCURL_USE_NGHTTP2=ON \
    -DCURL_USE_NGTCP2=ON \
    -DOPENSSL_ROOT_DIR="$PREFIX"

cmake_build "$PROJECT_ROOT"
