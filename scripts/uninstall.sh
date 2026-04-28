#!/usr/bin/env bash
set -euo pipefail

readonly SCRIPT_NAME="uninstall.sh"

PREFIX="${PREFIX:-$HOME/.local/bin}"
PURGE=false

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
Hi 卸载脚本

用法:
    bash scripts/uninstall.sh [选项]

选项:
    --prefix <dir>   自定义安装目录 (默认: $HOME/.local/bin)
                     可通过 PREFIX 环境变量覆盖
    --purge          删除 ~/.hione 目录 (用户主目录下的全局设置)
    --help           打印此帮助信息

示例:
    # 默认卸载 (保留 ~/.hione 数据)
    bash scripts/uninstall.sh

    # 完全清理 (删除用户主目录下的 ~/.hione)
    bash scripts/uninstall.sh --purge

注意:
    项目目录下的 .hione 不会被删除，这些是项目的会话记录。
EOF
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --prefix)
                PREFIX="$2"
                shift 2
                ;;
            --purge)
                PURGE=true
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

remove_binaries() {
    log_step "Removing binaries from $PREFIX..."
    
    local binaries=("hi" "hi-monitor" "hi-tauri")
    local removed=0
    local skipped=0
    
    for bin in "${binaries[@]}"; do
        local path="$PREFIX/$bin"
        
        if [[ -f "$path" ]]; then
            rm -f "$path"
            log_info "Removed: $path"
            ((removed++)) || true
        else
            log_info "Skipped (not found): $path"
            ((skipped++)) || true
        fi
    done
    
    printf '\n'
    log_info "Removed: $removed, Skipped: $skipped"
}

ask_hione_purge() {
    if [[ "$PURGE" == true ]]; then
        return 0
    fi
    
    if [[ ! -t 0 ]]; then
        log_info "Keeping ~/.hione data (no --purge specified, non-interactive mode)"
        return 1
    fi
    
    printf '\n'
    printf 'Do you want to remove ~/.hione (global settings)? [y/N] '
    read -r answer
    
    case "$answer" in
        y|Y|yes|YES)
            PURGE=true
            return 0
            ;;
        *)
            log_info "Keeping ~/.hione data"
            return 1
            ;;
    esac
}

purge_hione_dirs() {
    if [[ "$PURGE" != true ]]; then
        return
    fi
    
    log_step "Removing ~/.hione..."
    
    local hione_dir="$HOME/.hione"
    
    if [[ -d "$hione_dir" ]]; then
        rm -rf "$hione_dir"
        log_info "Removed: $hione_dir"
    else
        log_info "~/.hione not found"
    fi
    
    log_info "Project .hione directories are preserved (session records)"
}

print_summary() {
    printf '\n'
    log_info "$(color_bold 'Uninstall complete')"
}

main() {
    parse_args "$@"
    
    log_info "$(color_bold 'Hi Uninstall Script')"
    log_info "Platform: $(uname)"
    log_info "Install prefix: $PREFIX"
    printf '\n'
    
    remove_binaries
    
    if ask_hione_purge; then
        purge_hione_dirs
    fi
    
    print_summary
}

main "$@"