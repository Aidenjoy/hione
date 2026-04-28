#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_NAME="install.sh"
readonly PROJECT_NAME="hi"

MIN_RUST_VERSION="1.80"
MIN_NODE_VERSION="20"

PREFIX="${PREFIX:-$HOME/.local/bin}"
SKIP_FRONTEND=false
WITH_DESKTOP=false

has_tty() {
    [[ -t 1 ]]
}

color_red()    { if has_tty; then printf '\033[0;31m%s\033[0m' "$1"; else printf '%s' "$1"; fi }
color_green()  { if has_tty; then printf '\033[0;32m%s\033[0m' "$1"; else printf '%s' "$1"; fi }
color_yellow() { if has_tty; then printf '\033[0;33m%s\033[0m' "$1"; else printf '%s' "$1"; fi }
color_bold()   { if has_tty; then printf '\033[1m%s\033[0m' "$1"; else printf '%s' "$1"; fi }

log_info()    { printf '%s\n' "$(color_green '[INFO]') $1"; }
log_warn()    { printf '%s\n' "$(color_yellow '[WARN]') $1"; }
log_error()   { printf '%s\n' "$(color_red '[ERROR]') $1"; }
log_step()    { printf '%s\n' "$(color_bold '[STEP]') $1"; }

print_help() {
    cat <<'EOF'
Hi 安装脚本 (macOS / Linux)

用法:
    bash scripts/install.sh [选项]

选项:
    --prefix <dir>       自定义安装目录 (默认: $HOME/.local/bin)
                         可通过 PREFIX 环境变量覆盖
    --skip-frontend      跳过 npm install + npm run build
                         (适用于开发者快速重装)
    --with-desktop       额外构建 Tauri 桌面应用包
                         (产物在 crates/hi-tauri/src-tauri/target/release/bundle/)
    --help               打印此帮助信息

示例:
    # 默认安装 (仅 CLI + monitor)
    bash scripts/install.sh

    # 指定安装目录
    bash scripts/install.sh --prefix /usr/local/bin

    # 构建桌面应用
    bash scripts/install.sh --with-desktop

    # 快速重装 (跳过前端)
    bash scripts/install.sh --skip-frontend

前置依赖:
    - Rust >= 1.80       https://rustup.rs
    - Node.js >= 20      https://nodejs.org 或 https://github.com/Schniz/fnm
    - Linux: webkit2gtk-4.1
        Debian/Ubuntu: sudo apt install libwebkit2gtk-4.1-dev
        Fedora:        sudo dnf install webkit2gtk4.1-devel

安装后请确保安装目录在 PATH 中:
    export PATH="$HOME/.local/bin:$PATH"
EOF
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --prefix)
                PREFIX="$2"
                shift 2
                ;;
            --skip-frontend)
                SKIP_FRONTEND=true
                shift
                ;;
            --with-desktop)
                WITH_DESKTOP=true
                shift
                ;;
            --help|-h)
                print_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                print_help
                exit 1
                ;;
        esac
    done
}

check_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        log_error "$1 not found. Please install $1 first."
        case "$1" in
            rustc|cargo)
                printf '  Install Rust: https://rustup.rs\n'
                ;;
            node|npm)
                printf '  Install Node.js: https://nodejs.org or https://github.com/Schniz/fnm\n'
                ;;
        esac
        exit 1
    fi
}

check_version() {
    local cmd="$1"
    local min="$2"
    local actual
    
    case "$cmd" in
        rustc)
            actual=$(rustc --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+' | head -1)
            ;;
        cargo)
            actual=$(cargo --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+' | head -1)
            ;;
        node)
            actual=$(node --version 2>/dev/null | grep -oE 'v[0-9]+' | sed 's/v//')
            ;;
        *)
            log_error "Unknown version check: $cmd"
            exit 1
            ;;
    esac
    
    if [[ -z "$actual" ]]; then
        log_error "Could not determine $cmd version"
        exit 1
    fi
    
    local actual_major=$(echo "$actual" | cut -d. -f1)
    local min_major=$(echo "$min" | cut -d. -f1)
    
    if [[ "$actual_major" -lt "$min_major" ]]; then
        log_error "$cmd version $actual is too old (minimum: $min)"
        exit 1
    fi
    
    log_info "$cmd version: $actual (>= $min OK)"
}

check_platform_deps() {
    if [[ "$(uname)" == "Linux" ]]; then
        if ! pkg-config --exists webkit2gtk-4.1 2>/dev/null; then
            log_error "webkit2gtk-4.1 not found."
            printf '  Debian/Ubuntu: sudo apt install libwebkit2gtk-4.1-dev\n'
            printf '  Fedora:        sudo dnf install webkit2gtk4.1-devel\n'
            exit 1
        fi
        log_info "webkit2gtk-4.1 found OK"
    fi
}

check_deps() {
    log_step "Checking dependencies..."
    
    check_command rustc
    check_command cargo
    check_command node
    check_command npm
    
    check_version rustc "$MIN_RUST_VERSION"
    check_version cargo "$MIN_RUST_VERSION"
    check_version node "$MIN_NODE_VERSION"
    
    check_platform_deps
}

build_frontend() {
    if [[ "$SKIP_FRONTEND" == true ]]; then
        log_info "Skipping frontend build (--skip-frontend)"
        return
    fi
    
    log_step "Building frontend..."
    
    cd crates/hi-tauri
    
    log_info "Running npm install..."
    npm install
    
    log_info "Running npm run build..."
    npm run build
    
    cd ../..
    
    log_info "Frontend build complete OK"
}

build_binaries() {
    log_step "Building Rust binaries..."
    
    cargo build --workspace --release
    
    log_info "Binaries built OK"
}

ensure_tauri_cli() {
    if cargo tauri --version &>/dev/null; then
        log_info "tauri-cli found OK ($(cargo tauri --version 2>/dev/null))"
        return
    fi
    log_info "tauri-cli not found, installing via cargo..."
    if ! cargo install tauri-cli --locked; then
        log_error "Failed to install tauri-cli."
        printf '  Try manually: cargo install tauri-cli --locked\n'
        exit 1
    fi
}

build_desktop() {
    if [[ "$WITH_DESKTOP" != true ]]; then
        return
    fi

    log_step "Building Tauri desktop application..."

    ensure_tauri_cli

    cargo tauri build

    log_info "Desktop application built OK"
    log_info "Bundle location: crates/hi-tauri/src-tauri/target/release/bundle/"
}

install_binaries() {
    log_step "Installing binaries to $PREFIX..."
    
    mkdir -p "$PREFIX"
    
    local binaries=("hi" "hi-monitor")
    
    for bin in "${binaries[@]}"; do
        local src="target/release/$bin"
        local dest="$PREFIX/$bin"
        
        if [[ ! -f "$src" ]]; then
            log_error "Binary not found: $src"
            exit 1
        fi
        
        cp -f "$src" "$dest"
        chmod +x "$dest"
        
        log_info "Installed: $dest"
    done
    
    if [[ "$WITH_DESKTOP" == true ]]; then
        local tauri_src="target/release/hi-tauri"
        if [[ -f "$tauri_src" ]]; then
            local tauri_dest="$PREFIX/hi-tauri"
            cp -f "$tauri_src" "$tauri_dest"
            chmod +x "$tauri_dest"
            log_info "Installed: $tauri_dest"
        fi
    fi
}

check_path() {
    if ! echo "$PATH" | grep -qE "(^|:)$PREFIX(:|$)"; then
        log_warn "$PREFIX is not in your PATH"
        printf '\n'
        printf '  Add to PATH (temporary):\n'
        printf '    export PATH="%s:$PATH"\n\n' "$PREFIX"
        printf '  Add to PATH (permanent, add to ~/.zshrc or ~/.bashrc):\n'
        printf '    echo '\''export PATH="%s:$PATH"'\'' >> ~/.zshrc\n' "$PREFIX"
        printf '    source ~/.zshrc\n'
    else
        log_info "$PREFIX is in PATH OK"
    fi
}

print_success() {
    printf '\n'
    log_info "$(color_bold 'Installation complete!')"
    printf '\n'
    printf '  Installed binaries:\n'
    printf '    %s/hi\n' "$PREFIX"
    printf '    %s/hi-monitor\n' "$PREFIX"
    if [[ "$WITH_DESKTOP" == true ]]; then
        printf '    %s/hi-tauri\n' "$PREFIX"
    fi
    printf '\n'
    printf '  Quick start:\n'
    printf '    hi start claude,opencode,gemini\n'
    printf '\n'
}

main() {
    parse_args "$@"
    
    log_info "$(color_bold 'Hi Installation Script')"
    log_info "Platform: $(uname)"
    log_info "Install prefix: $PREFIX"
    printf '\n'
    
    check_deps
    
    build_frontend
    build_binaries
    build_desktop
    
    install_binaries
    
    check_path
    
    print_success
}

main "$@"