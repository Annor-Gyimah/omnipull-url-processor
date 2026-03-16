#!/usr/bin/env python3
"""
Test script for Rust URL Processor
Tests various URLs to verify the processor works correctly
"""

import subprocess
import json
import sys
from pathlib import Path


def test_url(binary_path, url, expected_supported=True):
    """Test a single URL"""
    print(f"\n{'='*60}")
    print(f"Testing: {url}")
    print(f"{'='*60}")

    try:
        result = subprocess.run(
            [str(binary_path), url],
            capture_output=True,
            text=True,
            timeout=10
        )

        # Parse JSON output
        # Debug: include stderr and return code in case the binary failed silently
        if result.stderr:
            print("--- BINARY STDERR ---")
            print(result.stderr)
            print("--- END STDERR ---")
        print(f"Exit code: {result.returncode}")

        data = json.loads(result.stdout) if result.stdout and result.stdout.strip() else None
        if data is None:
            print("No JSON output from binary.")
            if result.returncode != 0:
                print("Binary exited with non-zero code and produced no JSON.")
            return False

        print(f"Status: {'✅ SUPPORTED' if data['is_supported'] else '⏩ FALLBACK'}")
        print(f"Exit Code: {result.returncode}")

        if data.get('error'):
            print(f"❌ Error: {data['error']}")
            return False

        if data['is_supported']:
            print(f"  Filename: {data['filename']}")
            print(f"  Size: {data['size']} bytes")
            print(f"  Content-Type: {data['content_type']}")
            print(f"  Processing Time: {data['processing_time_ms']}ms")
            print(f"  Status Code: {data['status_code']}")

        # Check if result matches expectation
        if data['is_supported'] == expected_supported:
            print("✅ Test PASSED")
            return True
        else:
            print(f"❌ Test FAILED (expected supported={expected_supported})")
            return False

    except subprocess.TimeoutExpired:
        print("❌ TIMEOUT")
        return False
    except json.JSONDecodeError as e:
        print(f"❌ JSON Parse Error: {e}")
        print(f"Output: {result.stdout}")
        return False
    except Exception as e:
        print(f"❌ Error: {e}")
        return False


def main():
    # Find binary
    script_dir = Path(__file__).parent
    binaries_dir = script_dir.parent / 'rust_url_processor' / 'target'

    # Determine platform
    if sys.platform == 'win32':
        binary = binaries_dir / 'debug' / 'omnipull-url-processor.exe'
    elif sys.platform == 'darwin':
        binary = binaries_dir / 'macos' / 'omnipull-url-processor'
    else:
        binary = binaries_dir / 'linux' / 'omnipull-url-processor'

    # Check if binary exists
    if not binary.exists():
        print(f"❌ Binary not found: {binary}")
        print("\nBuild the binary first:")
        print("  cd rust_url_processor")
        print("  ./build.sh")
        sys.exit(1)

    print("=" * 60)
    print("OmniPull URL Processor - Test Suite")
    print("=" * 60)
    print(f"\nBinary: {binary}")

    # Test URLs
    test_cases = [
        # Direct files - should be supported
        ("https://www.python.org/ftp/python/pymanager/python-manager-25.2.msix", True, "Direct image file"),
        ("https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe", True, "Direct binary file"),
        # ("https://file-examples.com/storage/fe1fbc32e156405ab27b01c/2017/10/file_example_JPG_1MB.jpg", True, "Direct JPG"),
        # ("https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe", True, "RUST-INIT.exe")

        # Streaming/Complex - should NOT be supported (fallback to yt-dlp)
        # ("https://www.youtube.com/watch?v=dQw4w9WgXcQ", False, "YouTube video"),
        # ("https://vimeo.com/123456789", False, "Vimeo video"),
        # ("https://www.google.com", False, "HTML page"),
    ]

    passed = 0
    failed = 0

    for url, expected_supported, description in test_cases:
        print(f"\n📝 Test Case: {description}")
        if test_url(binary, url, expected_supported):
            passed += 1
        else:
            failed += 1

    # Summary
    print("\n" + "=" * 60)
    print("Test Summary")
    print("=" * 60)
    print(f"✅ Passed: {passed}/{len(test_cases)}")
    print(f"❌ Failed: {failed}/{len(test_cases)}")

    if failed == 0:
        print("\n🎉 All tests passed!")
        sys.exit(0)
    else:
        print(f"\n⚠️  {failed} test(s) failed")
        sys.exit(1)


if __name__ == '__main__':
    main()
