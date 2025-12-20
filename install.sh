#!/bin/bash

RED='\033[0;31m'
YELLOW='\033[1;32m'


cargo build --release

echo -e ${YELLOW}copy content

sudo install -Dm 644 assets/logo.png /usr/share/icons/hicolor/256x256/apps/io.codeberg.tahoso.azul_box.png

sudo install -Dm 755 target/release/azul-box /usr/bin/azulbox

sudo install -Dm 644 desktop/azul_box.desktop /usr/share/applications/io.codeberg.tahoso.azul_box.desktop

echo -e ${RED}Remember to install dependencies! Check https://codeberg.org/Tahoso/azul-box#dependencies for more info
