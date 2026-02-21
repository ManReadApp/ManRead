#!/usr/bin/env python3
"""
Convert a `.mrmang` export bundle to a folder structure:

{name}/
  info.json
  covers/...
  art/...
  chapters/{chapter}/{version}/...

`info.json` is compatible with `build_export_from_folder.py`.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import struct
import sys
from typing import NoReturn

MANGA_MAGIC = b"MRMANG01"


def fail(message: str) -> NoReturn:
    raise SystemExit(f"error: {message}")


def sanitize_component(value: str, fallback: str) -> str:
    cleaned = value.strip()
    cleaned = re.sub(r"[\\/:*?\"<>|]+", "_", cleaned)
    cleaned = cleaned.strip(". ").strip()
    return cleaned or fallback


def format_chapter_value(value: float) -> str:
    if float(value).is_integer():
        return str(int(value))
    text = f"{value:.8f}".rstrip("0").rstrip(".")
    return text or "0"


def detect_ext(data: bytes) -> str:
    if data.startswith(b"\x89PNG\r\n\x1a\n"):
        return "png"
    if len(data) >= 3 and data[0:3] == b"\xff\xd8\xff":
        return "jpg"
    if data.startswith(b"GIF87a") or data.startswith(b"GIF89a"):
        return "gif"
    if len(data) >= 12 and data[0:4] == b"RIFF" and data[8:12] == b"WEBP":
        return "webp"
    if data.startswith(b"BM"):
        return "bmp"
    if data.startswith(b"qoif"):
        return "qoi"
    if len(data) >= 12 and data[4:8] == b"ftyp":
        brand = data[8:12]
        if brand in {b"avif", b"avis"}:
            return "avif"
    return "bin"


def read_u32(data: bytes, offset: int) -> tuple[int, int]:
    if offset + 4 > len(data):
        fail("unexpected EOF while reading u32")
    return struct.unpack_from("<I", data, offset)[0], offset + 4


def read_varint(data: bytes, offset: int) -> tuple[int, int]:
    shift = 0
    out = 0
    while True:
        if offset >= len(data):
            fail("unexpected EOF while reading varint")
        byte = data[offset]
        offset += 1
        out |= (byte & 0x7F) << shift
        if (byte & 0x80) == 0:
            return out, offset
        shift += 7
        if shift > 70:
            fail("invalid varint (too large)")


def parse_fields(message: bytes) -> dict[int, list[tuple[int, object]]]:
    out: dict[int, list[tuple[int, object]]] = {}
    cursor = 0
    while cursor < len(message):
        key, cursor = read_varint(message, cursor)
        field = key >> 3
        wire = key & 0x07
        if wire == 0:
            value, cursor = read_varint(message, cursor)
        elif wire == 1:
            if cursor + 8 > len(message):
                fail("unexpected EOF in fixed64 field")
            value = struct.unpack_from("<Q", message, cursor)[0]
            cursor += 8
        elif wire == 2:
            length, cursor = read_varint(message, cursor)
            end = cursor + length
            if end > len(message):
                fail("unexpected EOF in length-delimited field")
            value = message[cursor:end]
            cursor = end
        elif wire == 5:
            if cursor + 4 > len(message):
                fail("unexpected EOF in fixed32 field")
            value = struct.unpack_from("<I", message, cursor)[0]
            cursor += 4
        else:
            fail(f"unsupported protobuf wire type: {wire}")
        out.setdefault(field, []).append((wire, value))
    return out


def values_as_strings(fields: dict[int, list[tuple[int, object]]], field: int) -> list[str]:
    out: list[str] = []
    for wire, value in fields.get(field, []):
        if wire != 2:
            fail(f"field {field} expected string wire type")
        out.append(bytes(value).decode("utf-8"))
    return out


def value_as_string(
    fields: dict[int, list[tuple[int, object]]], field: int, default: str = ""
) -> str:
    values = values_as_strings(fields, field)
    return values[0] if values else default


def value_as_optional_string(
    fields: dict[int, list[tuple[int, object]]], field: int
) -> str | None:
    values = values_as_strings(fields, field)
    return values[0] if values else None


def value_as_uint(
    fields: dict[int, list[tuple[int, object]]], field: int, default: int = 0
) -> int:
    values = fields.get(field, [])
    if not values:
        return default
    wire, value = values[0]
    if wire != 0:
        fail(f"field {field} expected uint varint")
    return int(value)


def value_as_double(
    fields: dict[int, list[tuple[int, object]]], field: int, default: float = 0.0
) -> float:
    values = fields.get(field, [])
    if not values:
        return default
    wire, value = values[0]
    if wire != 1:
        fail(f"field {field} expected double fixed64")
    return struct.unpack("<d", struct.pack("<Q", int(value)))[0]


def value_as_packed_u32(fields: dict[int, list[tuple[int, object]]], field: int) -> list[int]:
    out: list[int] = []
    for wire, value in fields.get(field, []):
        if wire == 0:
            out.append(int(value))
            continue
        if wire != 2:
            fail(f"field {field} expected packed uint32")
        blob = bytes(value)
        cursor = 0
        while cursor < len(blob):
            item, cursor = read_varint(blob, cursor)
            out.append(int(item))
    return out


def decode_string_list(blob: bytes) -> list[str]:
    return values_as_strings(parse_fields(blob), 1)


def decode_titles_entry(blob: bytes) -> tuple[str, list[str]]:
    fields = parse_fields(blob)
    key = value_as_string(fields, 1, "")
    values = []
    for wire, value in fields.get(2, []):
        if wire != 2:
            fail("titles map value has invalid wire type")
        values = decode_string_list(bytes(value))
    return key, values


def decode_tag(blob: bytes) -> dict[str, object]:
    fields = parse_fields(blob)
    return {
        "tag": value_as_string(fields, 1, ""),
        "description": value_as_optional_string(fields, 2),
        "sex": value_as_uint(fields, 3, 0),
    }


def decode_scraper(blob: bytes) -> dict[str, object]:
    fields = parse_fields(blob)
    return {
        "channel": value_as_string(fields, 1, ""),
        "url": value_as_string(fields, 2, ""),
        "enabled": bool(value_as_uint(fields, 3, 1)),
    }


def decode_volume(blob: bytes) -> dict[str, object]:
    fields = parse_fields(blob)
    return {
        "title": value_as_optional_string(fields, 1),
        "start": value_as_double(fields, 2, 0.0),
        "end": (
            value_as_double(fields, 3, 0.0)
            if fields.get(3)
            else None
        ),
    }


def decode_chapter_version(blob: bytes) -> dict[str, object]:
    fields = parse_fields(blob)
    return {
        "version": value_as_string(fields, 1, ""),
        "image_indexes": value_as_packed_u32(fields, 2),
        "link": value_as_optional_string(fields, 3),
    }


def decode_chapter(blob: bytes) -> dict[str, object]:
    fields = parse_fields(blob)
    versions: list[dict[str, object]] = []
    for wire, value in fields.get(6, []):
        if wire != 2:
            fail("chapter.version has invalid wire type")
        versions.append(decode_chapter_version(bytes(value)))

    tags: list[dict[str, object]] = []
    for wire, value in fields.get(3, []):
        if wire != 2:
            fail("chapter.tag has invalid wire type")
        tags.append(decode_tag(bytes(value)))

    return {
        "titles": values_as_strings(fields, 1),
        "chapter": value_as_double(fields, 2, 0.0),
        "tags": tags,
        "sources": values_as_strings(fields, 4),
        "release_date": value_as_optional_string(fields, 5),
        "versions": versions,
    }


def decode_metadata(blob: bytes) -> dict[str, object]:
    fields = parse_fields(blob)
    titles: dict[str, list[str]] = {}
    for wire, value in fields.get(1, []):
        if wire != 2:
            fail("metadata.titles has invalid wire type")
        language, values = decode_titles_entry(bytes(value))
        if language:
            titles[language] = values

    tags: list[dict[str, object]] = []
    for wire, value in fields.get(4, []):
        if wire != 2:
            fail("metadata.tags has invalid wire type")
        tags.append(decode_tag(bytes(value)))

    scraper: list[dict[str, object]] = []
    for wire, value in fields.get(12, []):
        if wire != 2:
            fail("metadata.scraper has invalid wire type")
        scraper.append(decode_scraper(bytes(value)))

    volumes: list[dict[str, object]] = []
    for wire, value in fields.get(13, []):
        if wire != 2:
            fail("metadata.volumes has invalid wire type")
        volumes.append(decode_volume(bytes(value)))

    chapters: list[dict[str, object]] = []
    for wire, value in fields.get(16, []):
        if wire != 2:
            fail("metadata.chapters has invalid wire type")
        chapters.append(decode_chapter(bytes(value)))

    return {
        "titles": titles,
        "kind": value_as_string(fields, 2, ""),
        "description": value_as_optional_string(fields, 3),
        "tags": tags,
        "status": value_as_uint(fields, 5, 0),
        "visibility": value_as_uint(fields, 6, 0),
        "uploader": value_as_string(fields, 7, ""),
        "artists": values_as_strings(fields, 8),
        "authors": values_as_strings(fields, 9),
        "publishers": values_as_strings(fields, 10),
        "sources": values_as_strings(fields, 11),
        "scraper": scraper,
        "volumes": volumes,
        "cover_image_indexes": value_as_packed_u32(fields, 14),
        "art_image_indexes": value_as_packed_u32(fields, 15),
        "chapters": chapters,
    }


def read_bundle(path: pathlib.Path) -> tuple[dict[str, object], list[bytes]]:
    payload = path.read_bytes()
    if len(payload) < 16:
        fail("bundle is too small")
    if payload[:8] != MANGA_MAGIC:
        fail("not a manga export bundle (magic mismatch)")

    cursor = 8
    metadata_len, cursor = read_u32(payload, cursor)
    metadata_end = cursor + metadata_len
    if metadata_end > len(payload):
        fail("metadata length exceeds bundle size")
    metadata_blob = payload[cursor:metadata_end]
    cursor = metadata_end

    image_count, cursor = read_u32(payload, cursor)
    images: list[bytes] = []
    for _ in range(image_count):
        image_len, cursor = read_u32(payload, cursor)
        image_end = cursor + image_len
        if image_end > len(payload):
            fail("image payload exceeds bundle size")
        images.append(payload[cursor:image_end])
        cursor = image_end
    if cursor != len(payload):
        fail("trailing bytes found after image payload")

    return decode_metadata(metadata_blob), images


def choose_name(metadata: dict[str, object], explicit: str | None) -> str:
    if explicit:
        return sanitize_component(explicit, "manga_export")
    titles = metadata.get("titles") or {}
    if isinstance(titles, dict):
        en = titles.get("en")
        if isinstance(en, list) and en:
            return sanitize_component(str(en[0]), "manga_export")
        for values in titles.values():
            if isinstance(values, list) and values:
                return sanitize_component(str(values[0]), "manga_export")
    return "manga_export"


def write_image(target: pathlib.Path, data: bytes) -> str:
    ext = detect_ext(data)
    target = target.with_suffix("." + ext)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data)
    return target.name


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Convert .mrmang export bundle to folder layout."
    )
    parser.add_argument("bundle", type=pathlib.Path, help="Input .mrmang bundle")
    parser.add_argument(
        "--out-dir",
        type=pathlib.Path,
        default=pathlib.Path("."),
        help="Output parent directory (default: current directory)",
    )
    parser.add_argument("--name", type=str, default=None, help="Output folder name")
    parser.add_argument(
        "--overwrite",
        action="store_true",
        help="Overwrite output folder if it exists",
    )
    args = parser.parse_args()

    bundle = args.bundle.resolve()
    if not bundle.exists() or not bundle.is_file():
        fail(f"bundle not found: {bundle}")

    metadata, images = read_bundle(bundle)
    out_dir = args.out_dir.resolve()
    out_dir.mkdir(parents=True, exist_ok=True)
    name = choose_name(metadata, args.name)
    root = out_dir / name

    if root.exists():
        if not args.overwrite:
            fail(f"output exists: {root} (use --overwrite)")
        if not root.is_dir():
            fail(f"output path exists and is not a directory: {root}")
    root.mkdir(parents=True, exist_ok=True)

    def image_at(index: int) -> bytes:
        if index < 0 or index >= len(images):
            fail(f"metadata references missing image index: {index}")
        return images[index]

    info = {
        "titles": metadata["titles"],
        "kind": metadata["kind"],
        "description": metadata["description"],
        "tags": metadata["tags"],
        "status": metadata["status"],
        "visibility": metadata["visibility"],
        "uploader": metadata["uploader"],
        "artists": metadata["artists"],
        "authors": metadata["authors"],
        "publishers": metadata["publishers"],
        "sources": metadata["sources"],
        "scraper": metadata["scraper"],
        "volumes": metadata["volumes"],
        "covers": [],
        "arts": [],
        "chapters": [],
    }

    for idx, image_index in enumerate(metadata["cover_image_indexes"]):
        base = root / "covers" / f"cover_{idx + 1:04d}"
        filename = write_image(base, image_at(int(image_index)))
        info["covers"].append(f"covers/{filename}")

    for idx, image_index in enumerate(metadata["art_image_indexes"]):
        base = root / "art" / f"art_{idx + 1:04d}"
        filename = write_image(base, image_at(int(image_index)))
        info["arts"].append(f"art/{filename}")

    chapter_name_counts: dict[str, int] = {}
    for chapter_idx, chapter in enumerate(metadata["chapters"], start=1):
        chapter_label = format_chapter_value(float(chapter["chapter"]))
        chapter_dir_name = sanitize_component(chapter_label, f"chapter_{chapter_idx:04d}")
        seen = chapter_name_counts.get(chapter_dir_name, 0)
        chapter_name_counts[chapter_dir_name] = seen + 1
        if seen > 0:
            chapter_dir_name = f"{chapter_dir_name}_{seen + 1}"

        version_name_counts: dict[str, int] = {}
        chapter_info = {
            "chapter": chapter["chapter"],
            "titles": chapter["titles"],
            "tags": chapter["tags"],
            "sources": chapter["sources"],
            "release_date": chapter["release_date"],
            "versions": [],
        }

        for version_idx, version in enumerate(chapter["versions"], start=1):
            version_name = sanitize_component(
                str(version.get("version") or ""),
                f"version_{version_idx:03d}",
            )
            version_seen = version_name_counts.get(version_name, 0)
            version_name_counts[version_name] = version_seen + 1
            if version_seen > 0:
                version_name = f"{version_name}_{version_seen + 1}"

            chapter_version_dir = root / "chapters" / chapter_dir_name / version_name
            chapter_version_dir.mkdir(parents=True, exist_ok=True)
            for page_idx, image_index in enumerate(version["image_indexes"], start=1):
                base = chapter_version_dir / f"page_{page_idx:04d}"
                write_image(base, image_at(int(image_index)))

            chapter_info["versions"].append(
                {
                    "version": version.get("version"),
                    "link": version.get("link"),
                    "path": f"chapters/{chapter_dir_name}/{version_name}",
                }
            )

        info["chapters"].append(chapter_info)

    info_path = root / "info.json"
    info_path.write_text(json.dumps(info, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    print(f"wrote: {root}")
    print(f"info: {info_path}")
    print(f"images: {len(images)}")
    print(f"chapters: {len(metadata['chapters'])}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
