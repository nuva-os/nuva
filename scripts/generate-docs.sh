#!/bin/bash
# Nuva OS
#
# Copyright (C) 2026 Nuva OS Team
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.


# Nuva OS Documentation Generation Script
#
# This script generates comprehensive documentation for Nuva OS
# including API docs, architecture docs, and user guides.

set -e

echo "=== Nuva OS Documentation Generation ==="
echo ""

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOCS_DIR="$PROJECT_ROOT/docs"
API_DIR="$DOCS_DIR/api"
ARCH_DIR="$DOCS_DIR/architecture"
GUIDES_DIR="$DOCS_DIR/guides"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    echo -e "${GREEN}[OK]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

print_error() {
    echo -e "${RED}[X]${NC} $1"
}

# Check dependencies
check_dependencies() {
    echo "Checking dependencies..."

    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Rust."
        exit 1
    fi
    print_status "Cargo found"

    if ! command -v mdbook &> /dev/null; then
        print_warning "mdBook not found. Installing..."
        cargo install mdbook
    fi
    print_status "mdBook found"

    if ! command -v rustdoc &> /dev/null; then
        print_error "rustdoc not found."
        exit 1
    fi
    print_status "rustdoc found"
}

# Generate API documentation
generate_api_docs() {
    echo ""
    echo "=== Generating API Documentation ==="

    mkdir -p "$API_DIR"

    # Generate Rust API documentation
    echo "Generating Rust API docs..."
    cargo doc --no-deps --target-dir "$API_DIR/target"

    # Generate documentation for all features
    echo "Generating docs for all features..."
    cargo doc --no-deps --all-features --target-dir "$API_DIR/target-all"

    # Generate private items documentation
    echo "Generating private items docs..."
    cargo doc --no-deps --document-private-items --target-dir "$API_DIR/target-private"

    print_status "API documentation generated"
}

# Generate architecture documentation
generate_arch_docs() {
    echo ""
    echo "=== Generating Architecture Documentation ==="

    mkdir -p "$ARCH_DIR"

    # Check if architecture docs exist
    if [ -d "$ARCH_DIR" ]; then
        print_status "Architecture documentation found"
    else
        print_warning "No architecture documentation found"
    fi

    # Generate architecture diagrams
    if command -v mermaid-cli &> /dev/null; then
        echo "Generating architecture diagrams..."
        find "$ARCH_DIR" -name "*.md" -exec mmdc -i {} -o {}.svg \;
        print_status "Architecture diagrams generated"
    else
        print_warning "mermaid-cli not found, skipping diagram generation"
    fi
}

# Generate user guides
generate_guides() {
    echo ""
    echo "=== Generating User Guides ==="

    mkdir -p "$GUIDES_DIR"

    # Build mdBook if book.toml exists
    if [ -f "$GUIDES_DIR/book.toml" ]; then
        echo "Building user guide with mdBook..."
        mdbook build "$GUIDES_DIR"
        print_status "User guide built"
    else
        print_warning "No book.toml found, skipping mdBook build"
    fi
}

# Generate documentation index
generate_index() {
    echo ""
    echo "=== Generating Documentation Index ==="

    INDEX_FILE="$DOCS_DIR/index.html"

    cat > "$INDEX_FILE" << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Nuva OS Documentation</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background: #f5f5f5;
        }
        h1 {
            color: #2c3e50;
            border-bottom: 3px solid #3498db;
            padding-bottom: 10px;
        }
        .section {
            background: white;
            padding: 20px;
            margin: 20px 0;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .section h2 {
            color: #34495e;
            margin-top: 0;
        }
        .links {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 15px;
            margin-top: 20px;
        }
        .link {
            display: block;
            padding: 15px;
            background: #3498db;
            color: white;
            text-decoration: none;
            border-radius: 5px;
            transition: background 0.3s;
        }
        .link:hover {
            background: #2980b9;
        }
        .link h3 {
            margin: 0 0 5px 0;
        }
        .link p {
            margin: 0;
            opacity: 0.9;
        }
    </style>
</head>
<body>
    <h1>Nuva OS Documentation Center</h1>

    <div class="section">
        <h2>API Documentation</h2>
        <div class="links">
            <a href="api/target/doc/nuva_os/index.html" class="link">
                <h3>Rust API</h3>
                <p>Core Rust API documentation</p>
            </a>
            <a href="api/target-all/doc/nuva_os/index.html" class="link">
                <h3>Full API</h3>
                <p>API documentation with all features</p>
            </a>
            <a href="api/target-private/doc/nuva_os/index.html" class="link">
                <h3>Internal API</h3>
                <p>Complete documentation including private items</p>
            </a>
        </div>
    </div>

    <div class="section">
        <h2>Architecture Documentation</h2>
        <div class="links">
            <a href="architecture/reorganization-plan.md" class="link">
                <h3>Architecture Reorganization</h3>
                <p>Detailed project architecture reorganization plan</p>
            </a>
            <a href="architecture/layer-rules.md" class="link">
                <h3>Layer Architecture Rules</h3>
                <p>Layer boundary and dependency rule definitions</p>
            </a>
            <a href="architecture/reorganization-summary.md" class="link">
                <h3>Reorganization Summary</h3>
                <p>Architecture reorganization results summary</p>
            </a>
        </div>
    </div>

    <div class="section">
        <h2>User Guides</h2>
        <div class="links">
            <a href="guides/book/index.html" class="link">
                <h3>User Manual</h3>
                <p>Nuva OS usage guide</p>
            </a>
            <a href="guides/quick-start.md" class="link">
                <h3>Quick Start</h3>
                <p>Quick start guide</p>
            </a>
        </div>
    </div>

    <div class="section">
        <h2>Documentation Standards</h2>
        <div class="links">
            <a href="standards/documentation-standard.md" class="link">
                <h3>Documentation Standard</h3>
                <p>Module documentation writing conventions</p>
            </a>
        </div>
    </div>
</body>
</html>
EOF

    print_status "Documentation index generated"
}

# Check documentation quality
check_docs_quality() {
    echo ""
    echo "=== Checking Documentation Quality ==="

    # Check for missing documentation
    echo "Checking for missing documentation..."
    if cargo clippy -- -W missing_docs 2>&1 | grep -q "warning: missing documentation"; then
        print_warning "Some items are missing documentation"
    else
        print_status "All items have documentation"
    fi

    # Check documentation examples
    echo "Checking documentation examples..."
    if cargo test --doc 2>&1 | grep -q "test result: ok"; then
        print_status "All documentation examples pass"
    else
        print_warning "Some documentation examples failed"
    fi
}

# Main function
main() {
    echo "Starting documentation generation..."
    echo ""

    check_dependencies
    generate_api_docs
    generate_arch_docs
    generate_guides
    generate_index
    check_docs_quality

    echo ""
    echo "=== Documentation Generation Complete ==="
    echo ""
    echo "Documentation has been generated in: $DOCS_DIR"
    echo ""
    echo "To view the documentation:"
    echo "  - API docs: open $API_DIR/target/doc/nuva_os/index.html"
    echo "  - Index: open $DOCS_DIR/index.html"
    echo ""
    print_status "Done!"
}

# Run main function
main "$@"
