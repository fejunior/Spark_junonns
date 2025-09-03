#!/bin/bash

# Build script for OpenFire Rust Authentication Library

set -e

echo "🦀 Building OpenFire Rust Authentication Library"
echo "================================================"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check if Java is installed
if ! command -v javac &> /dev/null; then
    echo "❌ Java compiler not found. Please install JDK 8 or later"
    exit 1
fi

echo "✅ Rust version: $(rustc --version)"
echo "✅ Java version: $(javac -version 2>&1)"
echo ""

# Build Rust library
echo "🔨 Building Rust library..."
cd rust-openfire-auth

echo "📦 Checking dependencies..."
cargo check

echo "🧪 Running tests..."
cargo test

echo "🚀 Building release version..."
cargo build --release

echo "✅ Rust library built successfully!"
echo "📍 Library location: target/release/libopenfire_auth.so (Linux) or .dylib (macOS) or .dll (Windows)"

cd ..

# Build Java interface (if gson is available)
echo ""
echo "🔨 Building Java interface..."
cd java-interface

# Create build directory
mkdir -p build/classes

# Try to compile Java files
echo "📦 Compiling Java interface..."
if javac -d build/classes src/main/java/org/jivesoftware/spark/openfire/*.java 2>/dev/null; then
    echo "✅ Java interface compiled successfully!"
    echo "📍 Classes location: java-interface/build/classes/"
else
    echo "⚠️  Java compilation requires Gson library. Skipping Java build."
    echo "   To build Java interface, add gson.jar to classpath:"
    echo "   javac -cp gson.jar -d build/classes src/main/java/org/jivesoftware/spark/openfire/*.java"
fi

cd ..

echo ""
echo "🎉 Build completed!"
echo ""
echo "Next steps:"
echo "1. Copy the native library to your Java library path"
echo "2. Add the Java classes to your classpath"
echo "3. Initialize the library with OpenFireAuthClient.initialize()"
echo ""
echo "Example usage:"
echo "  OpenFireAuthClient.Config config = new OpenFireAuthClient.Config(\"server\", \"domain\");"
echo "  OpenFireAuthClient client = new OpenFireAuthClient(config);"
echo "  AuthResult result = client.connect(\"user\", \"pass\", \"domain\");"
echo ""