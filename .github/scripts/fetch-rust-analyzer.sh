#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 2 ]; then
  echo "Usage: $0 <rust-target-triple> <output-dir>" >&2
  exit 1
fi

TARGET="$1"
OUT_DIR="$2"
mkdir -p "$OUT_DIR"

TMP_DIR="$(mktemp -d)"
RA_TAG="${RUST_ANALYZER_TAG:-latest}"

# Windows MSVC 预编译从 gzip 改为 zip（旧 .gz 在 release 上为 404）
if [[ "${TARGET}" == *"pc-windows-msvc"* ]]; then
  EXT="zip"
else
  EXT="gz"
fi

if [[ "${RA_TAG}" == "latest" ]]; then
  URL="https://github.com/rust-lang/rust-analyzer/releases/latest/download/rust-analyzer-${TARGET}.${EXT}"
else
  URL="https://github.com/rust-lang/rust-analyzer/releases/download/${RA_TAG}/rust-analyzer-${TARGET}.${EXT}"
fi

ARCHIVE_NAME="rust-analyzer-${TARGET}.${EXT}"
ARCHIVE_PATH="${TMP_DIR}/${ARCHIVE_NAME}"

echo "Downloading ${URL}"
curl -fL --retry 5 --retry-delay 2 --connect-timeout 15 --max-time 600 \
  -o "${ARCHIVE_PATH}" "${URL}"

python3 - <<'PY' "${ARCHIVE_PATH}" "${OUT_DIR}" "${TARGET}"
import gzip
import os
import stat
import sys
import zipfile

archive_path = sys.argv[1]
out_dir = sys.argv[2]
target = sys.argv[3]

is_win = "pc-windows-msvc" in target
binary_name = "rust-analyzer.exe" if is_win else "rust-analyzer"
destination = os.path.join(out_dir, binary_name)

if archive_path.endswith(".gz"):
    with gzip.open(archive_path, "rb") as src, open(destination, "wb") as dst:
        dst.write(src.read())
elif archive_path.endswith(".zip"):
    with zipfile.ZipFile(archive_path, "r") as zf:
        names = [
            n
            for n in zf.namelist()
            if n.endswith("rust-analyzer.exe") or n.endswith("/rust-analyzer.exe")
        ]
        if not names:
            raise SystemExit(f"no rust-analyzer.exe in zip: {archive_path}")
        names.sort(key=lambda n: (n.count("/"), len(n)))
        with open(destination, "wb") as dst:
            dst.write(zf.read(names[0]))
else:
    raise SystemExit(f"unsupported archive type: {archive_path}")

if not is_win:
    mode = os.stat(destination).st_mode
    os.chmod(destination, mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)

print(destination)
PY

rm -rf "${TMP_DIR}"
echo "rust-analyzer bundled into ${OUT_DIR}"
