#!/bin/sh

set -xe

TARGET="x86_64-unknown-linux-gnu"
PREFIX="$PROJECT_ROOT/build/$TARGET"
BUILD="$PREFIX/artifact"

cmake_build()
{
    PROJECT_DIR=$1
    PROJECT="${PROJECT_DIR%/cmake}"
    PROJECT="${PROJECT_DIR%/build/cmake}"
    PROJECT=$(basename "$PROJECT")
    ARTIFACT="$BUILD/$PROJECT"

    env \
        PKG_CONFIG_PATH="$PREFIX/lib/pkgconfig" \
    cmake \
        -B "$ARTIFACT" \
        -DCMAKE_BUILD_TYPE="Release" \
        -DBUILD_SHARED_LIBS=OFF \
        -DBUILD_STATIC_LIBS=ON \
        -DCMAKE_PREFIX_PATH="$PREFIX" \
        -DCMAKE_INSTALL_PREFIX="$PREFIX" \
        -DCMAKE_MODULE_PATH="$PREFIX/lib/cmake;$PREFIX/lib64/cmake" \
        ${@:2} "$PROJECT_DIR"

    env \
        PKG_CONFIG_PATH="$PREFIX/lib/pkgconfig" \
    cmake \
        --build "$ARTIFACT" \
        --parallel $(( $(nproc) * 2 )) \
        --target install
}

config_ssl()
{
    if ! [ -d "$BUILD/openssl" ]; then
        mkdir --parents "$BUILD/openssl"
        pushd "$BUILD/openssl"
        perl "$DEPS/openssl/Configure" \
            --prefix="$PREFIX" \
            --libdir=lib \
            no-shared \
            no-docs \
            no-demos \
            no-h3demo \
            no-tests

        popd
    fi
}

build_launcher()
{
    make \
        -C "$PROJECT_ROOT" \
        -j $(( $(nproc) * 2 )) \
        INSTALL_PREFIX="$PREFIX" \
        ARTIFACTS="$BUILD" \
        install
}
