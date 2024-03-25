#!/usr/bin/env bash

PRIVATE_KEY_PATH="aws-credential-manager.key";
if test -f "$PRIVATE_KEY_PATH"; then
    export TAURI_PRIVATE_KEY=$(cat $PRIVATE_KEY_PATH); # if the private key is stored on disk
    export TAURI_KEY_PASSWORD="";
    echo "private key loaded"
else
    echo "Warning: Private Key File Not Found";
fi
