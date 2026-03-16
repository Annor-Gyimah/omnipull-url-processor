#!/bin/bash
#####################################################################################
# Build script for OmniPull URL Processor
# Builds optimized release binary for current platform
#####################################################################################

set -e  # Exit on error

echo "=========================================="
echo "OmniPull URL Processor - Build Script"
echo "=========================================="
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Rust is not installed"
    echo ""
    echo "Install Rust from: https://rustup.rs/"
    echo "Or run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✅ Rust version: $(rustc --version)"
echo ""

# Detect platform
if [[ "$OSTYPE" == "darwin"* ]]; then
    PLATFORM="macOS"
    TARGET_DIR="../binaries/macos"
    BINARY_NAME="omnipull-url-processor"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    PLATFORM="Windows"
    TARGET_DIR="../binaries/windows"
    BINARY_NAME="omnipull-url-processor.exe"
else
    PLATFORM="Linux"
    TARGET_DIR="../binaries/linux"
    BINARY_NAME="omnipull-url-processor"
fi

echo "🖥️  Platform: $PLATFORM"
echo "📁 Target: $TARGET_DIR"
echo ""

# Clean previous build
echo "🧹 Cleaning previous build..."
cargo clean
echo ""

# Build release binary
echo "🔨 Building release binary..."
echo "   This may take a few minutes on first build..."
echo ""
cargo build --release

if [ $? -ne 0 ]; then
    echo ""
    echo "❌ Build failed!"
    exit 1
fi

echo ""
echo "✅ Build successful!"
echo ""

# Create binaries directory
echo "📁 Creating binaries directory..."
mkdir -p "$TARGET_DIR"

# Copy binary
echo "📦 Copying binary to $TARGET_DIR..."
cp "target/release/$BINARY_NAME" "$TARGET_DIR/"

# Make executable (Unix only)
if [[ "$OSTYPE" != "msys" && "$OSTYPE" != "win32" ]]; then
    chmod +x "$TARGET_DIR/$BINARY_NAME"
fi

echo ""
echo "=========================================="
echo "✅ Build Complete!"
echo "=========================================="
echo ""
echo "Binary location: $TARGET_DIR/$BINARY_NAME"
echo ""

# Test the binary
echo "🧪 Testing binary..."
"$TARGET_DIR/$BINARY_NAME" --help > /dev/null 2>&1

if [ $? -eq 0 ]; then
    echo "✅ Binary test passed!"
else
    echo "⚠️  Binary test failed (but binary was created)"
fi

echo ""
echo "🎉 Done! You can now use the Rust URL processor."
echo ""
echo "Try it:"
echo "  $TARGET_DIR/$BINARY_NAME \"https://httpbin.org/image/png\""
echo ""
