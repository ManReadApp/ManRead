#!/usr/bin/env python3
"""
Encode custom storage containers.

Formats:
- Chapter: MRCHAP01 + image blobs
- Manga:   MRMANG01 + bincode metadata blob + image blobs
"""

from __future__ import annotations

import argparse
import pathlib
import struct
import sys


CHAPTER_MAGIC = b"MRCHAP01"
MANGA_MAGIC = b"MRMANG01"


def read_file(path: pathlib.Path) -> bytes:
    return path.read_bytes()


def write_blob(out: bytearray, blob: bytes) -> None:
    out.extend(struct.pack("<I", len(blob)))
    out.extend(blob)


def encode_chapter(images: list[pathlib.Path], out_path: pathlib.Path) -> None:
    payload = bytearray()
    payload.extend(CHAPTER_MAGIC)
    payload.extend(struct.pack("<I", len(images)))
    for img in images:
        write_blob(payload, read_file(img))
    out_path.write_bytes(payload)


def encode_manga(
    metadata_bincode: pathlib.Path, images: list[pathlib.Path], out_path: pathlib.Path
) -> None:
    metadata = read_file(metadata_bincode)
    payload = bytearray()
    payload.extend(MANGA_MAGIC)
    write_blob(payload, metadata)
    payload.extend(struct.pack("<I", len(images)))
    for img in images:
        write_blob(payload, read_file(img))
    out_path.write_bytes(payload)


def main() -> int:
    parser = argparse.ArgumentParser(description="Encode chapter/manga containers")
    sub = parser.add_subparsers(dest="kind", required=True)

    p_ch = sub.add_parser("chapter", help="encode chapter images")
    p_ch.add_argument("--out", required=True, type=pathlib.Path)
    p_ch.add_argument("images", nargs="+", type=pathlib.Path)

    p_ma = sub.add_parser("manga", help="encode manga metadata + images")
    p_ma.add_argument("--out", required=True, type=pathlib.Path)
    p_ma.add_argument("--meta-bin", required=True, type=pathlib.Path)
    p_ma.add_argument("images", nargs="+", type=pathlib.Path)

    args = parser.parse_args()
    if args.kind == "chapter":
        encode_chapter(args.images, args.out)
    else:
        encode_manga(args.meta_bin, args.images, args.out)
    return 0


if __name__ == "__main__":
    sys.exit(main())
