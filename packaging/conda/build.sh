#!/bin/bash
set -eux

mkdir -p "${PREFIX}/bin"
install -Dm755 "${SRC_DIR}/zerostack" "${PREFIX}/bin/zerostack"
install -Dm644 "${SRC_DIR}/LICENSE" "${PREFIX}/share/licenses/${PKG_NAME}/LICENSE"
