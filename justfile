# Noah Wallet Justfile

set positional-arguments

# Default recipe to display available commands
default:
    @just --list

# Install dependencies
install:
    cd react-native-nitro-ark && yarn install

# Start Expo dev server
start:
    cd react-native-nitro-ark && yarn example start

# Android builds (regtest)
android:
    cd react-native-nitro-ark && yarn example android

# iOS builds (regtest)
ios:
    cd react-native-nitro-ark && yarn example ios

# iOS pod install
ios-prebuild:
    cd react-native-nitro-ark && yarn example ios:prebuild

# Clean commands
clean:
    cd react-native-nitro-ark && yarn clean

# Local regtest environment commands
setup-everything:
    ./scripts/ark-dev.sh setup-everything

up:
    ./scripts/ark-dev.sh up

down:
    ./scripts/ark-dev.sh down

stop:
    ./scripts/ark-dev.sh stop

# Ark dev shortcuts - pass arguments directly
bark *args:
    ./scripts/ark-dev.sh bark "$@"

aspd *args:
    ./scripts/ark-dev.sh aspd "$@"

bcli *args:
    ./scripts/ark-dev.sh bcli "$@"

lncli *args:
    ./scripts/ark-dev.sh lncli "$@"

cln *args:
    ./scripts/ark-dev.sh cln "$@"

# Bitcoin commands
create-wallet:
    ./scripts/ark-dev.sh create-wallet

create-bark-wallet:
    ./scripts/ark-dev.sh create-bark-wallet

generate blocks="101":
    ./scripts/ark-dev.sh generate {{blocks}}

fund-aspd amount:
    ./scripts/ark-dev.sh fund-aspd {{amount}}

send-to address amount:
    ./scripts/ark-dev.sh send-to {{address}} {{amount}}

# Lightning commands
setup-lightning:
    ./scripts/ark-dev.sh setup-lightning-channels

# Bark-cpp builds
build-android:
    cd bark-cpp && ./build-android.sh

build-ios:
    cd bark-cpp && ./build-ios.sh
