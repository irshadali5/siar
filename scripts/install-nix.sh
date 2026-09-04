#!/usr/bin/env bash
# ==============================================================================
# SIAR - Universal Cross-Distribution Nix Installer
# ==============================================================================
# Installs Nix package manager on any Linux distribution (Arch, Ubuntu, Debian,
# Fedora, RHEL, CentOS, Rocky, openSUSE, Alpine, Void, etc.) with Flakes enabled,
# modern multi-user daemon configuration, and SIAR development environment setup.
#
# Supported Init Systems: systemd, OpenRC, runit, WSL, Docker/containers
# Supported Architectures: x86_64, aarch64 (ARM64), armv7l
# ==============================================================================

set -euo pipefail

# Version & Defaults
INSTALLER_VERSION="1.0.0"
ENGINE="determinate"         # 'determinate' (recommended), 'official', or 'distro'
INSTALL_MODE="multi-user"     # 'multi-user' or 'single-user'
NON_INTERACTIVE=false        # set to true via -y / --yes
DRY_RUN=false
VERBOSE=false
ACTION="install"             # 'install', 'uninstall', 'status', 'doctor', 'enable-flakes'
BUILD_SIAR=false
INSTALL_SIAR=false
CHECK_SIAR=false

# Formatting / Colors (disabled if not terminal or NO_COLOR set)
if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
    BOLD="\033[1m"
    DIM="\033[2m"
    RED="\033[0;31m"
    GREEN="\033[0;32m"
    YELLOW="\033[0;33m"
    BLUE="\033[0;34m"
    MAGENTA="\033[0;35m"
    CYAN="\033[0;36m"
    NC="\033[0m" # No Color
else
    BOLD=""
    DIM=""
    RED=""
    GREEN=""
    YELLOW=""
    BLUE=""
    MAGENTA=""
    CYAN=""
    NC=""
fi

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

log_step() {
    echo -e "\n${BOLD}${CYAN}==>${NC} ${BOLD}$*${NC}"
}

print_banner() {
    cat << "EOF"
  ____ ___    _    ____     _   _ _         ___           _        _ _           
 / ___|_ _|  / \  |  _ \   | \ | (_)_  __  |_ _|_ __  ___| |_ __ _| | | ___ _ __ 
 \___ \| |  / _ \ | |_) |  |  \| | \ \/ /   | || '_ \/ __| __/ _` | | |/ _ \ '__|
  ___) | | / ___ \|  _ <   | |\  | |>  <    | || | | \__ \ || (_| | | |  __/ |   
 |____/___/_/   \_\_| \_\  |_| \_|_/_/\_\  |___|_| |_|___/\__\__,_|_|_|\___|_|   
                                                                                  
EOF
    echo -e "${DIM}SIAR Cross-Distribution Nix Installer v${INSTALLER_VERSION}${NC}\n"
}

print_help() {
    cat << EOF
SIAR Universal Nix Installer v${INSTALLER_VERSION}

Usage: $(basename "$0") [OPTIONS]

Installs and configures Nix package manager on any Linux distribution with
modern Nix Flakes, nix-command, and SIAR development environment support.

Options:
  -y, --yes              Non-interactive mode (automatic yes to all prompts)
  --engine <name>        Installation engine:
                           determinate (default, recommended: handles flakes,
                                        SELinux, Ubuntu 24.04 AppArmor, uninstaller)
                           official    (standard upstream NixOS multi-user installer)
                           distro      (use native package manager: pacman, apk)
  --multi-user           Force multi-user daemon installation (default when sudo/root available)
  --single-user          Force single-user / rootless installation (for restricted containers)
  --enable-flakes        Only configure Flakes and nix-command on an existing Nix installation
  --status               Check current Nix installation status, daemon health, and config
  --doctor               Run comprehensive verification diagnostics on Nix and Flakes
  --build-siar           Build SIAR workspace binaries using Nix after installation
  --install-siar         Install SIAR CLI directly into user's active Nix profile
  --check                Run 'nix flake check' on SIAR repository
  --uninstall            Cleanly uninstall Nix and remove all associated files and services
  --dry-run              Print the commands that would be executed without running them
  -v, --verbose          Enable verbose logging
  -h, --help             Display this help message and exit

Examples:
  # Quick installation on current Linux distribution
  ./install-nix.sh

  # Non-interactive installation (for scripts or CI)
  ./install-nix.sh -y

  # Install and immediately build SIAR workspace
  ./install-nix.sh --build-siar

  # Run diagnostic checks on existing Nix setup
  ./install-nix.sh --doctor

  # Clean uninstallation
  ./install-nix.sh --uninstall
EOF
}

# ==============================================================================
# Parse Command Line Arguments
# ==============================================================================
while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            print_banner
            print_help
            exit 0
            ;;
        -y|--yes|--non-interactive|--batch)
            NON_INTERACTIVE=true
            shift
            ;;
        --engine)
            if [[ -z "${2:-}" ]]; then
                log_error "Missing argument for --engine"
                exit 1
            fi
            ENGINE="$2"
            shift 2
            ;;
        --multi-user)
            INSTALL_MODE="multi-user"
            shift
            ;;
        --single-user)
            INSTALL_MODE="single-user"
            shift
            ;;
        --enable-flakes)
            ACTION="enable-flakes"
            shift
            ;;
        --status)
            ACTION="status"
            shift
            ;;
        --doctor)
            ACTION="doctor"
            shift
            ;;
        --uninstall)
            ACTION="uninstall"
            shift
            ;;
        --build-siar)
            BUILD_SIAR=true
            shift
            ;;
        --install-siar)
            INSTALL_SIAR=true
            shift
            ;;
        --check)
            CHECK_SIAR=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            echo "Use '$0 --help' for usage instructions."
            exit 1
            ;;
    esac
done

# ==============================================================================
# Privilege & Command Execution Utilities
# ==============================================================================
can_sudo() {
    if [[ $EUID -eq 0 ]]; then
        return 0
    fi
    if command -v sudo >/dev/null 2>&1; then
        return 0
    fi
    return 1
}

run_cmd() {
    if [[ "$DRY_RUN" == "true" ]]; then
        echo -e "${DIM}[DRY-RUN]${NC} $*"
        return 0
    fi
    if [[ "$VERBOSE" == "true" ]]; then
        log_info "Executing: $*"
    fi
    "$@"
}

run_privileged() {
    if [[ "$DRY_RUN" == "true" ]]; then
        echo -e "${DIM}[DRY-RUN privileged]${NC} $*"
        return 0
    fi
    if [[ $EUID -eq 0 ]]; then
        "$@"
    elif command -v sudo >/dev/null 2>&1; then
        sudo "$@"
    else
        log_error "Root privileges or 'sudo' required to execute: $*"
        exit 1
    fi
}

prompt_user() {
    local prompt_msg="$1"
    local default_ans="${2:-Y}"
    if [[ "$NON_INTERACTIVE" == "true" || "$DRY_RUN" == "true" ]]; then
        return 0
    fi

    local reply
    if [[ "$default_ans" =~ ^[Yy]$ ]]; then
        read -r -p "$(echo -e "${BOLD}${prompt_msg} [Y/n]: ${NC}")" reply
        reply="${reply:-y}"
        [[ "$reply" =~ ^[Yy]$ ]] && return 0 || return 1
    else
        read -r -p "$(echo -e "${BOLD}${prompt_msg} [y/N]: ${NC}")" reply
        reply="${reply:-n}"
        [[ "$reply" =~ ^[Yy]$ ]] && return 0 || return 1
    fi
}

# ==============================================================================
# System & Distribution Detection
# ==============================================================================
detect_environment() {
    OS_NAME="$(uname -s)"
    OS_ARCH="$(uname -m)"
    DISTRO_ID="unknown"
    DISTRO_NAME="Generic Linux"
    DISTRO_FAMILY="unknown"
    INIT_SYSTEM="unknown"
    IS_WSL=false
    IS_CONTAINER=false

    # Architecture Normalization
    case "$OS_ARCH" in
        x86_64|amd64)
            NIX_SYSTEM="x86_64-linux"
            ;;
        aarch64|arm64)
            NIX_SYSTEM="aarch64-linux"
            ;;
        armv7l|armv7)
            NIX_SYSTEM="armv7l-linux"
            ;;
        i686|i386)
            NIX_SYSTEM="i686-linux"
            ;;
        *)
            NIX_SYSTEM="${OS_ARCH}-linux"
            ;;
    esac

    # Darwin / macOS check
    if [[ "$OS_NAME" == "Darwin" ]]; then
        if [[ "$OS_ARCH" == "arm64" ]]; then
            NIX_SYSTEM="aarch64-darwin"
        else
            NIX_SYSTEM="x86_64-darwin"
        fi
        DISTRO_ID="macos"
        DISTRO_NAME="macOS $(sw_vers -productVersion 2>/dev/null || true)"
        DISTRO_FAMILY="darwin"
        INIT_SYSTEM="launchd"
        return 0
    fi

    # Container / WSL check
    if [[ -f /.dockerenv ]] || [[ -f /run/.containerenv ]] || grep -qa 'docker\|lxc\|containerd' /proc/1/cgroup 2>/dev/null; then
        IS_CONTAINER=true
    fi
    if grep -qi 'microsoft' /proc/version 2>/dev/null; then
        IS_WSL=true
    fi

    # Init System
    if [[ -d /run/systemd/system ]] || pidof systemd >/dev/null 2>&1; then
        INIT_SYSTEM="systemd"
    elif [[ -f /sbin/openrc ]] || [[ -d /run/openrc ]]; then
        INIT_SYSTEM="openrc"
    elif [[ -d /run/runit ]] || command -v runit >/dev/null 2>&1; then
        INIT_SYSTEM="runit"
    else
        INIT_SYSTEM="sysvinit-or-container"
    fi

    # Distribution parsing via /etc/os-release
    if [[ -f /etc/os-release ]]; then
        # shellcheck disable=SC1091
        source /etc/os-release
        DISTRO_ID="${ID:-unknown}"
        DISTRO_NAME="${PRETTY_NAME:-$NAME}"

        local id_like="${ID_LIKE:-}"
        case "$DISTRO_ID" in
            arch|manjaro|endeavouros|garuda|artix)
                DISTRO_FAMILY="arch"
                ;;
            ubuntu|debian|linuxmint|pop|elementary|kali|raspbian|devuan)
                DISTRO_FAMILY="debian"
                ;;
            fedora|rhel|centos|rocky|almalinux|oracle|nobara)
                DISTRO_FAMILY="rhel"
                ;;
            opensuse*|sles|suse)
                DISTRO_FAMILY="suse"
                ;;
            alpine)
                DISTRO_FAMILY="alpine"
                ;;
            void)
                DISTRO_FAMILY="void"
                ;;
            gentoo)
                DISTRO_FAMILY="gentoo"
                ;;
            *)
                if [[ "$id_like" =~ (arch) ]]; then
                    DISTRO_FAMILY="arch"
                elif [[ "$id_like" =~ (debian|ubuntu) ]]; then
                    DISTRO_FAMILY="debian"
                elif [[ "$id_like" =~ (rhel|fedora|centos) ]]; then
                    DISTRO_FAMILY="rhel"
                elif [[ "$id_like" =~ (suse) ]]; then
                    DISTRO_FAMILY="suse"
                else
                    DISTRO_FAMILY="generic"
                fi
                ;;
        esac
    fi
}

print_system_info() {
    log_info "Detected System Information:"
    echo -e "  • ${BOLD}Distribution:${NC}   ${DISTRO_NAME} (${DISTRO_ID})"
    echo -e "  • ${BOLD}Distro Family:${NC}  ${DISTRO_FAMILY}"
    echo -e "  • ${BOLD}Architecture:${NC}   ${OS_ARCH} (Target: ${NIX_SYSTEM})"
    echo -e "  • ${BOLD}Init System:${NC}    ${INIT_SYSTEM}"
    if [[ "$IS_WSL" == "true" ]]; then
        echo -e "  • ${BOLD}Environment:${NC}    WSL (Windows Subsystem for Linux)"
    elif [[ "$IS_CONTAINER" == "true" ]]; then
        echo -e "  • ${BOLD}Environment:${NC}    Container (Docker/Podman/LXC)"
    fi
    echo ""
}

# ==============================================================================
# Prerequisites & Package Manager Assistance
# ==============================================================================
ensure_prerequisites() {
    log_step "Checking System Prerequisites"

    local missing_tools=()
    for tool in curl tar xz; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            missing_tools+=("$tool")
        fi
    done

    if [[ ${#missing_tools[@]} -eq 0 ]]; then
        log_success "Core prerequisite tools present: curl, tar, xz."
        return 0
    fi

    log_warn "Missing required tools: ${missing_tools[*]}"
    if ! prompt_user "Install missing tools using native package manager (${DISTRO_FAMILY})?" "y"; then
        log_error "Cannot proceed without: ${missing_tools[*]}"
        exit 1
    fi

    log_info "Installing missing dependencies..."
    case "$DISTRO_FAMILY" in
        arch)
            run_privileged pacman -Sy --needed --noconfirm "${missing_tools[@]}" sudo ca-certificates
            ;;
        debian)
            run_privileged apt-get update
            local deb_packages=()
            for t in "${missing_tools[@]}"; do
                [[ "$t" == "xz" ]] && deb_packages+=("xz-utils") || deb_packages+=("$t")
            done
            run_privileged apt-get install -y "${deb_packages[@]}" sudo ca-certificates
            ;;
        rhel)
            if command -v dnf >/dev/null 2>&1; then
                run_privileged dnf install -y "${missing_tools[@]}" sudo ca-certificates
            else
                run_privileged yum install -y "${missing_tools[@]}" sudo ca-certificates
            fi
            ;;
        suse)
            run_privileged zypper --non-interactive install "${missing_tools[@]}" sudo ca-certificates
            ;;
        alpine)
            local apk_packages=()
            for t in "${missing_tools[@]}"; do
                apk_packages+=("$t")
            done
            run_privileged apk add --no-cache "${apk_packages[@]}" sudo shadow ca-certificates
            ;;
        void)
            run_privileged xbps-install -Sy "${missing_tools[@]}" sudo ca-certificates
            ;;
        *)
            log_error "Automatic package installation not implemented for distro family: ${DISTRO_FAMILY}."
            log_error "Please manually install: ${missing_tools[*]} and re-run."
            exit 1
            ;;
    esac
    log_success "Prerequisites successfully installed."
}

# ==============================================================================
# Existing Installation Checks
# ==============================================================================
is_nix_installed() {
    if command -v nix >/dev/null 2>&1 || [[ -d /nix/store && -d /nix/var/nix ]]; then
        return 0
    fi
    return 1
}

source_nix_env() {
    # Attempt to load Nix environment in current shell session
    if [[ -f "/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh" ]]; then
        # shellcheck disable=SC1091
        source "/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh" || true
    elif [[ -f "${HOME}/.nix-profile/etc/profile.d/nix.sh" ]]; then
        # shellcheck disable=SC1091
        source "${HOME}/.nix-profile/etc/profile.d/nix.sh" || true
    elif [[ -f "/etc/profile.d/nix.sh" ]]; then
        # shellcheck disable=SC1091
        source "/etc/profile.d/nix.sh" || true
    fi

    # Append standard Nix paths to PATH if missing
    if [[ ":$PATH:" != *":/nix/var/nix/profiles/default/bin:"* ]] && [[ -d "/nix/var/nix/profiles/default/bin" ]]; then
        export PATH="/nix/var/nix/profiles/default/bin:$PATH"
    fi
    if [[ -d "${HOME}/.nix-profile/bin" ]] && [[ ":$PATH:" != *":${HOME}/.nix-profile/bin:"* ]]; then
        export PATH="${HOME}/.nix-profile/bin:$PATH"
    fi
}

# ==============================================================================
# Flakes & Configuration Engine
# ==============================================================================
configure_nix_flakes() {
    log_step "Configuring Nix Flakes & Optimizations"

    local nix_conf_dir="/etc/nix"
    local nix_conf="${nix_conf_dir}/nix.conf"
    local current_user
    current_user="$(whoami)"

    if [[ "$INSTALL_MODE" == "single-user" && $EUID -ne 0 ]]; then
        nix_conf_dir="${HOME}/.config/nix"
        nix_conf="${nix_conf_dir}/nix.conf"
        mkdir -p "$nix_conf_dir"
    else
        run_privileged mkdir -p "$nix_conf_dir"
    fi

    local temp_conf
    temp_conf="$(mktemp)"

    # Base configuration block
    cat << EOF > "$temp_conf"
# Generated by SIAR Nix Installer
experimental-features = nix-command flakes
trusted-users = root @wheel @sudo ${current_user}
extra-substituters = https://cache.nixos.org https://nix-community.cachix.org
extra-trusted-public-keys = cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY= nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs=
max-jobs = auto
cores = 0
keep-outputs = true
keep-derivations = true
EOF

    if [[ "$DRY_RUN" == "true" ]]; then
        echo -e "${DIM}[DRY-RUN] Writing to ${nix_conf}:${NC}"
        cat "$temp_conf"
        rm -f "$temp_conf"
        return 0
    fi

    if [[ -f "$nix_conf" ]]; then
        log_info "Existing ${nix_conf} found. Updating settings..."
        # Backup existing
        if [[ "$INSTALL_MODE" == "single-user" && $EUID -ne 0 ]]; then
            cp "$nix_conf" "${nix_conf}.backup-$(date +%s)"
        else
            run_privileged cp "$nix_conf" "${nix_conf}.backup-$(date +%s)"
        fi

        # Check for experimental-features
        if grep -q "experimental-features" "$nix_conf"; then
            if ! grep -q "flakes" "$nix_conf"; then
                run_privileged sed -i 's/experimental-features.*/& flakes/' "$nix_conf"
            fi
            if ! grep -q "nix-command" "$nix_conf"; then
                run_privileged sed -i 's/experimental-features.*/& nix-command/' "$nix_conf"
            fi
        else
            echo "experimental-features = nix-command flakes" | run_privileged tee -a "$nix_conf" >/dev/null
        fi

        # Ensure trusted-users exists
        if ! grep -q "trusted-users" "$nix_conf"; then
            echo "trusted-users = root @wheel @sudo ${current_user}" | run_privileged tee -a "$nix_conf" >/dev/null
        fi
        rm -f "$temp_conf"
    else
        log_info "Creating new ${nix_conf}..."
        if [[ "$INSTALL_MODE" == "single-user" && $EUID -ne 0 ]]; then
            mv "$temp_conf" "$nix_conf"
        else
            run_privileged mv "$temp_conf" "$nix_conf"
            run_privileged chmod 644 "$nix_conf"
        fi
    fi

    # Restart nix-daemon if running under systemd
    if [[ "$INIT_SYSTEM" == "systemd" ]] && systemctl is-active nix-daemon.service >/dev/null 2>&1; then
        log_info "Restarting nix-daemon to apply configuration changes..."
        run_privileged systemctl restart nix-daemon.service || true
    fi

    log_success "Flakes and modern CLI enabled in ${nix_conf}."
}

configure_shell_profile() {
    log_step "Configuring Shell Profiles"

    local current_user_home="${HOME}"
    local daemon_hook='/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh'
    local single_hook="${current_user_home}/.nix-profile/etc/profile.d/nix.sh"

    local hook_cmd=""
    if [[ -f "$daemon_hook" ]]; then
        hook_cmd="[ -e '${daemon_hook}' ] && . '${daemon_hook}'"
    elif [[ -f "$single_hook" ]]; then
        hook_cmd="[ -e '${single_hook}' ] && . '${single_hook}'"
    fi

    if [[ -n "$hook_cmd" ]]; then
        for rc in "${current_user_home}/.bashrc" "${current_user_home}/.zshrc" "${current_user_home}/.profile"; do
            if [[ -f "$rc" ]]; then
                if ! grep -q "nix-daemon.sh\|nix.sh" "$rc"; then
                    log_info "Adding Nix environment hook to ${rc}"
                    if [[ "$DRY_RUN" == "true" ]]; then
                        echo -e "${DIM}[DRY-RUN] append '${hook_cmd}' to ${rc}${NC}"
                    else
                        echo -e "\n# Nix Package Manager environment hook\n${hook_cmd}" >> "$rc"
                    fi
                fi
            fi
        done
    fi

    source_nix_env
    log_success "Shell profiles verified."
}

# ==============================================================================
# Installation Engines
# ==============================================================================
install_determinate() {
    log_step "Installing Nix via Determinate Systems Installer"
    log_info "Determinate Systems installer is the modern, battle-tested standard with out-of-the-box"
    log_info "Flakes support, multi-user daemon, Ubuntu 24.04 AppArmor, and SELinux compatibility."

    local installer_url="https://install.determinate.systems/nix"
    local installer_args=("install" "linux")

    if [[ "$NON_INTERACTIVE" == "true" ]]; then
        installer_args+=("--no-confirm")
    fi

    if [[ "$INIT_SYSTEM" != "systemd" ]]; then
        log_warn "Non-systemd init system detected (${INIT_SYSTEM}). Configuring with --init none..."
        installer_args+=("--init" "none")
    fi

    local current_user
    current_user="$(whoami)"
    installer_args+=(
        "--extra-conf" "experimental-features = nix-command flakes"
        "--extra-conf" "trusted-users = root @wheel @sudo ${current_user}"
        "--extra-conf" "extra-substituters = https://cache.nixos.org https://nix-community.cachix.org"
        "--extra-conf" "extra-trusted-public-keys = cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY= nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    )

    log_info "Fetching and launching Determinate installer..."
    if [[ "$DRY_RUN" == "true" ]]; then
        echo -e "${DIM}[DRY-RUN] curl --proto '=https' --tlsv1.2 -sSf -L ${installer_url} | sh -s -- ${installer_args[*]}${NC}"
        return 0
    fi

    curl --proto '=https' --tlsv1.2 -sSf -L "$installer_url" | sh -s -- "${installer_args[@]}"
    log_success "Determinate Nix installer finished."
}

install_official() {
    log_step "Installing Nix via Upstream Official NixOS Installer"
    local installer_url="https://nixos.org/nix/install"
    local installer_args=()

    if [[ "$INSTALL_MODE" == "multi-user" ]]; then
        installer_args+=("--daemon")
    else
        installer_args+=("--no-daemon")
    fi

    if [[ "$NON_INTERACTIVE" == "true" ]]; then
        installer_args+=("--no-channel-add")
    fi

    log_info "Fetching upstream official Nix installer..."
    if [[ "$DRY_RUN" == "true" ]]; then
        echo -e "${DIM}[DRY-RUN] curl -sSf -L ${installer_url} | sh -s -- ${installer_args[*]}${NC}"
        return 0
    fi

    curl -sSf -L "$installer_url" | sh -s -- "${installer_args[@]}"
    log_success "Upstream official installer finished."
}

install_distro_native() {
    log_step "Installing Nix via Distro Native Package Manager (${DISTRO_FAMILY})"
    case "$DISTRO_FAMILY" in
        arch)
            log_info "Installing nix via pacman..."
            run_privileged pacman -Sy --needed --noconfirm nix
            if [[ "$INIT_SYSTEM" == "systemd" ]]; then
                run_privileged systemctl enable --now nix-daemon.service nix-daemon.socket
            fi
            ;;
        alpine)
            log_info "Installing nix via apk..."
            run_privileged apk add --no-cache nix
            if [[ "$INIT_SYSTEM" == "openrc" ]]; then
                run_privileged rc-update add nix-daemon default || true
                run_privileged rc-service nix-daemon start || true
            fi
            ;;
        *)
            log_error "Distro-native package installation not available for ${DISTRO_NAME}."
            log_info "Falling back to Determinate Systems installer..."
            install_determinate
            return
            ;;
    esac

    log_success "Distro-native Nix package installed."
}

# ==============================================================================
# Diagnostics, Status, and Doctor
# ==============================================================================
check_status() {
    print_banner
    detect_environment
    print_system_info
    source_nix_env

    log_step "SIAR Nix Status Check"

    if ! is_nix_installed; then
        log_warn "Nix is NOT currently installed or not detected in PATH / /nix."
        echo "Run './install-nix.sh' to install Nix."
        return 1
    fi

    local nix_ver
    nix_ver="$(nix --version 2>/dev/null || echo 'Unknown')"
    log_success "Nix executable found: ${BOLD}${nix_ver}${NC}"
    echo -e "  • ${BOLD}Binary Location:${NC} $(command -v nix || echo 'not in current PATH')"

    # Check Daemon status
    if [[ "$INIT_SYSTEM" == "systemd" ]]; then
        echo -n "  • Daemon Service:  "
        if systemctl is-active nix-daemon.service >/dev/null 2>&1; then
            echo -e "${GREEN}active (running)${NC}"
        else
            echo -e "${YELLOW}inactive / not running${NC}"
        fi
        echo -n "  • Daemon Socket:   "
        if systemctl is-active nix-daemon.socket >/dev/null 2>&1; then
            echo -e "${GREEN}active (listening)${NC}"
        else
            echo -e "${YELLOW}inactive${NC}"
        fi
    fi

    # Check Flakes Configuration
    echo -n "  • Flakes Support:  "
    if nix flake --help >/dev/null 2>&1; then
        echo -e "${GREEN}enabled & ready${NC}"
    else
        echo -e "${RED}disabled${NC} (Run './install-nix.sh --enable-flakes' to activate)"
    fi

    # Check Store Size
    if [[ -d /nix/store ]]; then
        local store_size
        store_size="$(du -sh /nix/store 2>/dev/null | cut -f1 || echo 'N/A')"
        echo -e "  • /nix/store Size: ${store_size}"
    fi

    echo ""
}

run_doctor() {
    print_banner
    detect_environment
    source_nix_env

    log_step "SIAR Nix Environment Doctor"

    local all_pass=true

    # 1. Binary check
    echo -n "  [1/5] Checking nix CLI in PATH... "
    if command -v nix >/dev/null 2>&1; then
        echo -e "${GREEN}PASS${NC} ($(nix --version))"
    else
        echo -e "${RED}FAIL${NC}"
        all_pass=false
    fi

    # 2. Nix evaluation test
    echo -n "  [2/5] Testing Nix expression evaluation (1 + 1)... "
    local eval_result
    eval_result="$(nix eval --expr "1 + 1" 2>/dev/null || echo "error")"
    if [[ "$eval_result" == "2" ]]; then
        echo -e "${GREEN}PASS${NC} (Evaluated successfully)"
    else
        echo -e "${RED}FAIL${NC} (Evaluation returned: ${eval_result})"
        all_pass=false
    fi

    # 3. Flakes feature test
    echo -n "  [3/5] Testing Nix Flakes command integration... "
    if nix flake --help >/dev/null 2>&1; then
        echo -e "${GREEN}PASS${NC}"
    else
        echo -e "${RED}FAIL${NC} (Flakes not enabled in nix.conf)"
        all_pass=false
    fi

    # 4. Multi-user socket / daemon connectivity
    echo -n "  [4/5] Testing connection to nix-daemon socket... "
    if [[ -e /nix/var/nix/daemon-socket/socket ]]; then
        echo -e "${GREEN}PASS${NC} (/nix/var/nix/daemon-socket/socket exists)"
    elif [[ "$INSTALL_MODE" == "single-user" ]]; then
        echo -e "${YELLOW}SKIP${NC} (Single-user mode)"
    else
        echo -e "${YELLOW}WARN${NC} (Socket file not found; daemon might be socket-activated)"
    fi

    # 5. SIAR Workspace Flake Check (if inside SIAR repository)
    if [[ -f "./flake.nix" ]]; then
        echo -n "  [5/5] Inspecting local SIAR flake.nix... "
        if nix flake metadata --no-update-lock-file >/dev/null 2>&1; then
            echo -e "${GREEN}PASS${NC} (SIAR flake is valid)"
        else
            echo -e "${YELLOW}WARN${NC} (Flake metadata check encountered an issue)"
        fi
    else
        echo -e "  [5/5] SIAR flake check: ${DIM}Skipped (Not in SIAR workspace root)${NC}"
    fi

    echo ""
    if [[ "$all_pass" == "true" ]]; then
        log_success "All critical diagnostic checks passed! Nix is ready for SIAR development."
    else
        log_warn "One or more checks did not pass. Run './install-nix.sh' to repair or configure."
    fi
}

# ==============================================================================
# Uninstaller
# ==============================================================================
uninstall_nix() {
    print_banner
    detect_environment
    log_step "Uninstalling Nix Package Manager"

    log_warn "This will remove Nix, /nix store, build users, and configuration from this system."
    if ! prompt_user "Are you absolutely sure you want to uninstall Nix?" "n"; then
        log_info "Uninstallation cancelled."
        exit 0
    fi

    # 1. Determinate receipt uninstaller (cleanest method if available)
    if [[ -x /nix/nix-installer ]]; then
        log_info "Detected Determinate Nix uninstaller receipt. Running /nix/nix-installer uninstall..."
        run_privileged /nix/nix-installer uninstall
        log_success "Determinate Nix uninstallation completed."
        return 0
    fi

    # 2. Universal Manual Purge
    log_info "Performing universal system cleanup..."

    # Stop and disable systemd services
    if [[ "$INIT_SYSTEM" == "systemd" ]]; then
        log_info "Stopping and disabling nix-daemon services..."
        run_privileged systemctl stop nix-daemon.service nix-daemon.socket 2>/dev/null || true
        run_privileged systemctl disable nix-daemon.service nix-daemon.socket 2>/dev/null || true
        run_privileged rm -f /etc/systemd/system/nix-daemon.service /etc/systemd/system/nix-daemon.socket
        run_privileged systemctl daemon-reload 2>/dev/null || true
    fi

    # Remove Nix profile hooks and configurations
    log_info "Removing configuration and profile scripts..."
    run_privileged rm -rf /etc/nix /etc/profile.d/nix.sh /etc/profile.d/nix-daemon.sh
    run_privileged rm -rf "${HOME}/.config/nix" "${HOME}/.nix-profile" "${HOME}/.nix-defexpr" "${HOME}/.nix-channels"

    # Remove nixbld users and group
    log_info "Removing nixbld users and groups..."
    for i in $(seq 1 32); do
        run_privileged userdel "nixbld${i}" 2>/dev/null || true
    done
    run_privileged groupdel nixbld 2>/dev/null || true

    # Remove /nix directory
    log_info "Removing /nix store and metadata directory..."
    run_privileged rm -rf /nix

    log_success "Nix has been completely removed from this system."
    echo "Tip: Check ~/.bashrc or ~/.zshrc if you wish to remove any custom environment exports."
}

# ==============================================================================
# SIAR Integration Operations
# ==============================================================================
perform_siar_operations() {
    source_nix_env

    if [[ "$CHECK_SIAR" == "true" ]]; then
        log_step "Running SIAR Flake Validation (nix flake check)"
        run_cmd nix flake check --print-build-logs
        log_success "SIAR flake checks passed."
    fi

    if [[ "$BUILD_SIAR" == "true" ]]; then
        log_step "Building SIAR CLI and Binaries via Nix Flake"
        log_info "Building target .#siar-cli..."
        run_cmd nix build .#siar-cli --print-build-logs
        log_success "SIAR CLI successfully built! Result located at ./result/bin/siar"
    fi

    if [[ "$INSTALL_SIAR" == "true" ]]; then
        log_step "Installing SIAR CLI to User Nix Profile"
        run_cmd nix profile install .#siar-cli
        log_success "SIAR CLI installed into profile! Run 'siar --help' to test."
    fi
}

# ==============================================================================
# Main Installer Routine
# ==============================================================================
main() {
    case "$ACTION" in
        status)
            check_status
            exit 0
            ;;
        doctor)
            run_doctor
            exit 0
            ;;
        uninstall)
            uninstall_nix
            exit 0
            ;;
        enable-flakes)
            detect_environment
            configure_nix_flakes
            configure_shell_profile
            run_doctor
            exit 0
            ;;
    esac

    print_banner
    detect_environment
    print_system_info

    # Check if Nix is already installed
    if is_nix_installed; then
        source_nix_env
        local ver
        ver="$(nix --version 2>/dev/null || echo 'Unknown')"
        log_info "Nix is already installed on this machine: ${BOLD}${ver}${NC}"
        
        if nix flake --help >/dev/null 2>&1; then
            log_success "Flakes and modern nix-command are already configured!"
        else
            log_warn "Nix is installed, but Flakes / nix-command are not yet enabled."
            if prompt_user "Enable Flakes and modern CLI features now?" "y"; then
                configure_nix_flakes
                configure_shell_profile
            fi
        fi

        perform_siar_operations
        log_success "SIAR Nix environment is ready!"
        exit 0
    fi

    # Confirm installation with user if interactive
    log_info "Ready to install Nix on ${DISTRO_NAME} (${OS_ARCH})."
    echo -e "  • Selected Engine:  ${BOLD}${ENGINE}${NC}"
    echo -e "  • Selected Mode:    ${BOLD}${INSTALL_MODE}${NC}"
    echo -e "  • Flakes Enabled:   ${BOLD}yes (automatic)${NC}"
    echo ""

    if ! prompt_user "Do you want to proceed with the Nix installation?" "y"; then
        log_info "Installation aborted by user."
        exit 0
    fi

    # Pre-flight check & dependency resolution
    ensure_prerequisites

    # Execute selected engine
    case "$ENGINE" in
        determinate)
            install_determinate
            ;;
        official)
            install_official
            ;;
        distro)
            install_distro_native
            ;;
        *)
            log_error "Unsupported engine: ${ENGINE}"
            exit 1
            ;;
    esac

    # Post-installation configuration
    source_nix_env
    configure_nix_flakes
    configure_shell_profile

    # Verification & Next Steps
    echo ""
    log_step "Installation Complete & Verification"
    if is_nix_installed; then
        local installed_ver
        installed_ver="$(nix --version 2>/dev/null || echo 'installed')"
        log_success "${BOLD}Nix (${installed_ver}) was installed successfully!${NC}"
    else
        log_warn "Nix installation completed, but 'nix' is not yet in the active shell path."
        log_info "Run this command to load it immediately in this terminal:"
        echo -e "  ${BOLD}. /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh${NC}"
    fi

    # Execute optional SIAR targets if requested
    perform_siar_operations

    echo ""
    echo -e "${BOLD}Next Steps with SIAR:${NC}"
    echo -e "  1. Activate Nix in your current shell session (or restart your terminal):"
    echo -e "     ${CYAN}source /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh${NC}"
    echo -e "  2. Enter the hermetic SIAR development shell with all dependencies:"
    echo -e "     ${CYAN}nix develop${NC}"
    echo -e "  3. Build SIAR CLI or desktop application:"
    echo -e "     ${CYAN}nix build .#siar-cli${NC}"
    echo -e "     ${CYAN}nix build .#siar-desktop${NC}"
    echo -e "     ${CYAN}nix build .#all${NC}"
    echo -e "  4. Run flake checks:"
    echo -e "     ${CYAN}nix flake check${NC}"
    echo ""
}

main "$@"
