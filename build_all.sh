#!/bin/bash

# Function to build for a specific platform
build_platform() {
    local platform=$1

    echo "Building for $platform..."

    mkdir -p "build/nrush-$platform" || { echo "Error: Failed to create directory for $platform build"; exit 1; }

    cargo_target=""
    binary_name=""

    case $platform in
        win64)
            cargo_target="x86_64-pc-windows-gnu"
            binary_name="nrush.exe"
            ;;
        win32)
            cargo_target="i686-pc-windows-gnu"
            binary_name="nrush.exe"
            ;;
        macos)
            cargo_target="x86_64-apple-darwin"
            binary_name="nrush"
            ;;
        linux)
            cargo_target="x86_64-unknown-linux-gnu"
            binary_name="nrush"
            ;;
        *)
            echo "Unsupported platform: $platform"
            exit 1
            ;;
    esac

    cargo build --release --target $cargo_target || { echo "Error: Build failed for $platform"; exit 1; }

    cp -f "target/$cargo_target/release/$binary_name" "build/nrush-$platform/" || { echo "Error: Failed to copy built binary to $platform build directory"; exit 1; }

    tar -C "build" -czvf "build/nrush-$platform.tar.gz" "nrush-$platform" || { echo "Error: Failed to compress $platform build"; exit 1; }

    rm -rf "build/nrush-$platform" || { echo "Error: Failed to remove $platform build directory"; exit 1; }
}

platforms=("win64" "win32" "linux")

for platform in "${platforms[@]}"; do
    build_platform "$platform"
done

echo "Build completed for all specified platforms."
