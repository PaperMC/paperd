#!/usr/bin/env bash

shopt -s nocasematch

function help_message() {
    echo "paperd installer script"
    echo
    echo "This script will install the paperd binary to your system. The current OS and versions supported are:"
    echo " * Ubuntu 16.04, 18.04, 19.10, 20.04"
    echo " * Debian Jessie (8), Stretch (9), Buster (10)"
    echo " * Fedora 32, 31"
    echo " * CentOS 8, 7"
    echo
    echo "By default this script will install paperd into /usr/bin and install any packages needed first."
    echo "paperd will ask before attempting to run any commands as root. The paperd binary will be installed"
    echo "with the owner being the same as the directory it's installed into, and with permissions 755."
    echo
    echo "Arguments:"
    echo "   --no-console               | Install paperd with no console support. This removes"
    echo "                              | the dependency on ncurses."
    echo "   --yes                      | Assume 'yes' answer to all questions, good for running"
    echo "                              | non-interactively in a script."
    echo "   --bin-location=<directory> | Install paperd to a custom directory other than /usr/bin."
    echo "   -h, --help                 | Print this help message"
    exit 0
}

function self_build() {
    echo "Please see <url> for instructions on how to build paperd yourself."
    exit 1
}

function print_detect_info() {
    echo "Detected system info:"
    echo "System: $1"
    echo "Version: $2"
    echo
}

function ask_okay() {
    if [[ "$1" == "true" ]] ; then
        echo "Assuming okay because --yes was passed"
        return
    fi
    read -r -n 1 -p "Is this okay? [yN] " val
    echo
    if [[ "$val" != "y" ]] ; then
        echo "Exiting"
        exit 1
    fi
}

function debian_package_exists() {
    dpkg -s "$1" &> /dev/null
    return
}

function fedora_package_exists() {
    [[ "$(rpm -qa "$1")" != "" ]]
    return
}

function setup_debian() {
    PACKAGES_NEEDED=""

    if ! debian_package_exists curl ; then
        PACKAGES_NEEDED="curl"
    fi
    if ! debian_package_exists tar ; then
        PACKAGES_NEEDED="$PACKAGES_NEEDED tar"
    fi
    if ! debian_package_exists xz-utils ; then
        PACKAGES_NEEDED="$PACKAGES_NEEDED xz-utils"
    fi
    if [[ "$1" == "true" ]] ; then
        if ! debian_package_exists libncurses5 ; then
            PACKAGES_NEEDED="$PACKAGES_NEEDED libncurses5"
        fi
    fi

    if [[ "$PACKAGES_NEEDED" != "" ]] ; then
        echo "In order to extract and use paperd, the following packages need to be installed: $PACKAGES_NEEDED"
        echo "The following command will be run:"
        echo
        echo "sudo apt-get install $PACKAGES_NEEDED"
        echo
        ask_okay "$2"

        # shellcheck disable=SC2086
        sudo apt-get install $PACKAGES_NEEDED
        echo
    fi
}

function setup_fedora() {
    PACKAGES_NEEDED=""

    if ! debian_package_exists curl ; then
        PACKAGES_NEEDED="curl"
    fi
    if ! debian_package_exists tar ; then
        PACKAGES_NEEDED="$PACKAGES_NEEDED tar"
    fi
    if ! debian_package_exists xz-utils ; then
        PACKAGES_NEEDED="$PACKAGES_NEEDED xz"
    fi
    if [[ "$1" == 1 ]] ; then
        if ! debian_package_exists libncurses5 ; then
            PACKAGES_NEEDED="$PACKAGES_NEEDED ncurses"
        fi
    fi

    if [[ "$PACKAGES_NEEDED" != "" ]] ; then
        echo "In order to extract and use paperd, the following packages need to be installed: $PACKAGES_NEEDED"
        echo "The following command will be run:"
        echo
        echo "sudo yum install $PACKAGES_NEEDED"
        echo
        ask_okay "$2"

        # shellcheck disable=SC2086
        sudo yum install $PACKAGES_NEEDED
        echo
    fi
}

SUPPORT_CONSOLE="true"
ALWAYS_YES="false"
INSTALL_DIRECTORY=/usr/bin

while [[ $# -gt 0 ]] ; do
    key="$1"
    case $key in
    --no-console)
        SUPPORT_CONSOLE="false"
        shift
        ;;
    --yes)
        ALWAYS_YES="true"
        shift
        ;;
    --bin-location)
        INSTALL_DIRECTORY="$2"
        shift
        shift
        ;;
    -h|--help)
        help_message
        ;;
    esac
done

if [[ ! -d "$INSTALL_DIRECTORY" ]] ; then
    echo "Error: $INSTALL_DIRECTORY does not exist, exiting."
    exit 1
fi

if [[ ! -f "/etc/os-release" ]] ; then
    echo "Error: Cannot determine your distro, which means you're probably using a distro which doesn't have a pre-built version available."
    self_build
fi

. /etc/os-release

SYSTEM_NAME=""
VERSION_NAME=""
PRETTY_SYSTEM_NAME=""
PRETTY_VERSION_NAME=""

case "$ID" in
"ubuntu")
    SYSTEM_NAME="ubuntu"
    PRETTY_SYSTEM_NAME="${SYSTEM_NAME^}"
    case "$VERSION_ID" in
    "20.04"*)
        VERSION_NAME="20.04"
        ;;
    "19.10"*)
        VERSION_NAME="19.10"
        ;;
    "18.04"*)
        VERSION_NAME="18.04"
        ;;
    "16.04"*)
        VERSION_NAME="16.04"
        ;;
    *)
        echo "Error: Unsupported version, only current Ubuntu versions are supported (20.04, 19.10, 18.04, 16.04)"
        self_build
        ;;
    esac
    PRETTY_VERSION_NAME="$VERSION_NAME"

    print_detect_info "$PRETTY_SYSTEM_NAME" "$PRETTY_VERSION_NAME"
    setup_debian "$SUPPORT_CONSOLE" "$ALWAYS_YES"
    ;;
"debian")
    SYSTEM_NAME="debian"
    PRETTY_SYSTEM_NAME="${SYSTEM_NAME^}"
    case "$VERSION_CODENAME" in
    "buster")
        VERSION_NAME="buster"
        ;;
    "stretch")
        VERSION_NAME="stretch"
        ;;
    "jessie")
        VERSION_NAME="jessie"
        ;;
    *)
        echo "Error: Unsupported version, only current Debian versions are supported (buster, stretch, jessie)"
        self_build
        ;;
    esac
    PRETTY_VERSION_NAME="${VERSION_NAME^}"

    print_detect_info "$PRETTY_SYSTEM_NAME" "$PRETTY_VERSION_NAME"
    setup_debian "$SUPPORT_CONSOLE" "$ALWAYS_YES"
    ;;
"fedora")
    SYSTEM_NAME="fedora"
    PRETTY_VERSION_NAME="${SYSTEM_NAME^}"
    case "$VERSION_ID" in
    "32"*)
        VERSION_NAME="32"
        ;;
    "31"*)
        VERSION_NAME="31"
        ;;
    *)
        echo "Error: Unsupported version, only current Fedora versions are supported (32, 31)"
        self_build
        ;;
    esac
    PRETTY_VERSION_NAME="$VERSION_NAME"

    print_detect_info "$PRETTY_SYSTEM_NAME" "$PRETTY_VERSION_NAME"
    setup_fedora "$SUPPORT_CONSOLE" "$ALWAYS_YES"
    ;;
"centos")
    SYSTEM_NAME="centos"
    PRETTY_SYSTEM_NAME="CentOS"
    case "$VERSION_ID" in
    "8"*)
        VERSION_NAME="8"
        ;;
    "7"*)
        VERSION_NAME="7"
        ;;
    *)
        echo "Error: Unsupported version, only current CentOS versions are supported (8, 7)"
        self_build
        ;;
    esac
    PRETTY_VERSION_NAME="$VERSION_NAME"

    print_detect_info "$PRETTY_SYSTEM_NAME" "$PRETTY_VERSION_NAME"
    setup_fedora "$SUPPORT_CONSOLE" "$ALWAYS_YES"
    ;;
*)
    echo "Error: Unsupported distro: $ID. Prebuild binaries are only available for Debian, Ubuntu, Fedora, or CentOS."
    self_build
    ;;
esac

URL_TARGET="paperd-$SYSTEM_NAME-$VERSION_NAME"
if [[ "$SUPPORT_CONSOLE" == "false" ]] ; then
    URL_TARGET="$URL_TARGET-no-console"
fi
URL_TARGET="$URL_TARGET.tar.xz"

OWNER_USER="$(stat -c "%U" "$INSTALL_DIRECTORY")"
OWNER_GROUP="$(stat -c "%G" "$INSTALL_DIRECTORY")"

echo "About to download https://dl.demonwav.com/paperd-$URL_TARGET"
ask_okay "$ALWAYS_YES"
echo

TMP_FILE_NAME="paperd_output_$RANDOM"
TMP_FILE_NAME_TAR="$TMP_FILE_NAME.tar.xz"
curl --proto '=https' --tlsv1.2 -fL -o "/tmp/$TMP_FILE_NAME_TAR" "https://dl.demonwav.com/$URL_TARGET"

mkdir "/tmp/$TMP_FILE_NAME"
tar fx "/tmp/$TMP_FILE_NAME_TAR" -C "/tmp/$TMP_FILE_NAME"
rm "/tmp/$TMP_FILE_NAME_TAR"

echo
echo "About to copy paperd to the destination directory and set permissions. The following commands will be run:"
echo
echo "sudo mv /tmp/$TMP_FILE_NAME/paperd $INSTALL_DIRECTORY/paperd"
echo "sudo chown $OWNER_USER:$OWNER_GROUP $INSTALL_DIRECTORY/paperd"
echo "sudo chmod 755 $INSTALL_DIRECTORY/paperd"
echo
ask_okay "$ALWAYS_YES"

sudo mv "/tmp/$TMP_FILE_NAME/paperd" "$INSTALL_DIRECTORY/paperd"
sudo chown "$OWNER_USER":"$OWNER_GROUP" "$INSTALL_DIRECTORY/paperd"
sudo chmod 755 "$INSTALL_DIRECTORY/paperd"
