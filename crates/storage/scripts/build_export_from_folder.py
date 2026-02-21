#!/usr/bin/env python3
"""
Build an `.mrmang` export bundle from a folder manifest.

The manifest is JSON. Paths are resolved relative to `folder`.

Minimal example:
{
  "titles": { "en": ["My Manga"] },
  "kind": "manga",
  "uploader": "my-user",
  "chapters": [
    {
      "chapter": 1.0,
      "titles": ["Chapter 1"],
      "versions": [
        { "version": "en", "path": "chapters/001/en" }
      ]
    }
  ]
}

Optional top-level fields:
- description, status, visibility, tags, artists, authors, publishers, sources, scraper, volumes
- covers: list/path for cover images
- art or arts: list/path for art images

Each chapter version supports:
- path: file, directory, or glob pattern
- images: explicit list of image paths
- link: optional string

Image ordering is natural-sorted (e.g. page2 before page10).
"""

from __future__ import annotations

import argparse
import glob
import json
import pathlib
import re
import struct
import sys
from typing import Iterable

MANGA_MAGIC = b"MRMANG01"
MAX_U32 = 0xFFFFFFFF
GLOB_CHARS = set("*?[]")


def fail(message: str) -> "NoReturn":
    raise SystemExit(f"error: {message}")


def natural_key(value: str) -> list[object]:
    parts = re.split(r"(\d+)", value.casefold())
    out: list[object] = []
    for part in parts:
        if part.isdigit():
            out.append(int(part))
        else:
            out.append(part)
    return out


def is_glob_pattern(value: str) -> bool:
    return any(char in GLOB_CHARS for char in value)


def resolve_path(root: pathlib.Path, value: str) -> pathlib.Path:
    raw = pathlib.Path(value)
    if raw.is_absolute():
        return raw
    return (root / raw).resolve()


def list_image_files_from_spec(root: pathlib.Path, spec: str) -> list[pathlib.Path]:
    spec_path = pathlib.Path(spec)
    if is_glob_pattern(spec):
        pattern = spec if spec_path.is_absolute() else str(root / spec)
        candidates = [pathlib.Path(item).resolve() for item in glob.glob(pattern)]
    else:
        resolved = resolve_path(root, spec)
        if not resolved.exists():
            fail(f"path does not exist: {spec}")
        if resolved.is_dir():
            candidates = [
                item.resolve()
                for item in resolved.iterdir()
                if item.is_file() and not item.name.startswith(".")
            ]
        elif resolved.is_file():
            candidates = [resolved]
        else:
            fail(f"path is neither file nor directory: {spec}")

    files = [item for item in candidates if item.is_file() and not item.name.startswith(".")]
    files.sort(key=lambda item: natural_key(str(item.name)))
    if not files:
        fail(f"image spec resolved to zero files: {spec}")
    return files


def collect_paths(root: pathlib.Path, value: object) -> list[pathlib.Path]:
    if value is None:
        return []
    if isinstance(value, str):
        return list_image_files_from_spec(root, value)
    if isinstance(value, list):
        out: list[pathlib.Path] = []
        for item in value:
            out.extend(collect_paths(root, item))
        return out
    fail("path value must be string, list, or null")


def encode_varint(value: int) -> bytes:
    if value < 0:
        fail("negative varint is not supported")
    out = bytearray()
    while value >= 0x80:
        out.append((value & 0x7F) | 0x80)
        value >>= 7
    out.append(value)
    return bytes(out)


def field_key(field_number: int, wire_type: int) -> bytes:
    return encode_varint((field_number << 3) | wire_type)


def encode_uint_field(field_number: int, value: int) -> bytes:
    return field_key(field_number, 0) + encode_varint(value)


def encode_bool_field(field_number: int, value: bool) -> bytes:
    return encode_uint_field(field_number, 1 if value else 0)


def encode_double_field(field_number: int, value: float) -> bytes:
    return field_key(field_number, 1) + struct.pack("<d", value)


def encode_bytes_field(field_number: int, value: bytes) -> bytes:
    return field_key(field_number, 2) + encode_varint(len(value)) + value


def encode_string_field(field_number: int, value: str) -> bytes:
    return encode_bytes_field(field_number, value.encode("utf-8"))


def encode_message_field(field_number: int, payload: bytes) -> bytes:
    return encode_bytes_field(field_number, payload)


def encode_packed_u32_field(field_number: int, values: Iterable[int]) -> bytes:
    encoded = bytearray()
    for value in values:
        encoded.extend(encode_varint(int(value)))
    if not encoded:
        return b""
    return encode_bytes_field(field_number, bytes(encoded))


def as_non_empty_str_list(value: object) -> list[str]:
    if value is None:
        return []
    if not isinstance(value, list):
        fail("expected a list of strings")
    out = []
    for item in value:
        if not isinstance(item, str):
            fail("string list contains non-string value")
        stripped = item.strip()
        if stripped:
            out.append(stripped)
    return out


def parse_tag_item(value: object) -> dict[str, object]:
    if isinstance(value, str):
        return {"tag": value.strip(), "description": None, "sex": 0}
    if isinstance(value, dict):
        tag_name = str(value.get("tag", "")).strip()
        if not tag_name:
            fail("tag object is missing 'tag'")
        description = value.get("description")
        if description is not None and not isinstance(description, str):
            fail("tag.description must be string or null")
        sex = value.get("sex", 0)
        try:
            sex_value = int(sex)
        except Exception as exc:  # pragma: no cover - defensive
            raise SystemExit(f"error: tag.sex must be an integer ({exc})") from exc
        return {"tag": tag_name, "description": description, "sex": sex_value}
    fail("tag must be string or object")


def encode_string_list(items: list[str]) -> bytes:
    out = bytearray()
    for item in items:
        out.extend(encode_string_field(1, item))
    return bytes(out)


def encode_titles_entry(language: str, items: list[str]) -> bytes:
    entry = bytearray()
    entry.extend(encode_string_field(1, language))
    entry.extend(encode_message_field(2, encode_string_list(items)))
    return bytes(entry)


def encode_tag(tag: dict[str, object]) -> bytes:
    out = bytearray()
    out.extend(encode_string_field(1, str(tag["tag"])))
    description = tag.get("description")
    if description:
        out.extend(encode_string_field(2, str(description)))
    out.extend(encode_uint_field(3, int(tag.get("sex", 0))))
    return bytes(out)


def encode_scraper(scraper: dict[str, object]) -> bytes:
    out = bytearray()
    out.extend(encode_string_field(1, str(scraper["channel"])))
    out.extend(encode_string_field(2, str(scraper["url"])))
    out.extend(encode_bool_field(3, bool(scraper.get("enabled", True))))
    return bytes(out)


def encode_volume(volume: dict[str, object]) -> bytes:
    out = bytearray()
    title = volume.get("title")
    if title:
        out.extend(encode_string_field(1, str(title)))
    out.extend(encode_double_field(2, float(volume["start"])))
    if volume.get("end") is not None:
        out.extend(encode_double_field(3, float(volume["end"])))
    return bytes(out)


def encode_chapter_version(version: dict[str, object]) -> bytes:
    out = bytearray()
    out.extend(encode_string_field(1, str(version["version"])))
    out.extend(encode_packed_u32_field(2, version["image_indexes"]))
    if version.get("link"):
        out.extend(encode_string_field(3, str(version["link"])))
    return bytes(out)


def encode_chapter(chapter: dict[str, object]) -> bytes:
    out = bytearray()
    for title in chapter["titles"]:
        out.extend(encode_string_field(1, title))
    out.extend(encode_double_field(2, float(chapter["chapter"])))
    for tag in chapter["tags"]:
        out.extend(encode_message_field(3, encode_tag(tag)))
    for source in chapter["sources"]:
        out.extend(encode_string_field(4, source))
    if chapter.get("release_date"):
        out.extend(encode_string_field(5, str(chapter["release_date"])))
    for version in chapter["versions"]:
        out.extend(encode_message_field(6, encode_chapter_version(version)))
    return bytes(out)


def build_metadata_bytes(metadata: dict[str, object]) -> bytes:
    out = bytearray()
    titles: dict[str, list[str]] = metadata["titles"]  # type: ignore[assignment]
    for language in sorted(titles.keys()):
        out.extend(
            encode_message_field(1, encode_titles_entry(language, titles[language]))
        )
    out.extend(encode_string_field(2, str(metadata["kind"])))
    if metadata.get("description"):
        out.extend(encode_string_field(3, str(metadata["description"])))
    for tag in metadata["tags"]:
        out.extend(encode_message_field(4, encode_tag(tag)))
    out.extend(encode_uint_field(5, int(metadata["status"])))
    out.extend(encode_uint_field(6, int(metadata["visibility"])))
    out.extend(encode_string_field(7, str(metadata["uploader"])))
    for item in metadata["artists"]:
        out.extend(encode_string_field(8, item))
    for item in metadata["authors"]:
        out.extend(encode_string_field(9, item))
    for item in metadata["publishers"]:
        out.extend(encode_string_field(10, item))
    for item in metadata["sources"]:
        out.extend(encode_string_field(11, item))
    for scraper in metadata["scraper"]:
        out.extend(encode_message_field(12, encode_scraper(scraper)))
    for volume in metadata["volumes"]:
        out.extend(encode_message_field(13, encode_volume(volume)))
    out.extend(encode_packed_u32_field(14, metadata["cover_image_indexes"]))
    out.extend(encode_packed_u32_field(15, metadata["art_image_indexes"]))
    for chapter in metadata["chapters"]:
        out.extend(encode_message_field(16, encode_chapter(chapter)))
    return bytes(out)


def normalize_titles(raw_titles: object) -> dict[str, list[str]]:
    if not isinstance(raw_titles, dict):
        fail("titles must be an object: {\"lang\": [\"title\"]}")
    out: dict[str, list[str]] = {}
    for lang, raw_values in raw_titles.items():
        if not isinstance(lang, str):
            fail("titles language key must be string")
        language = lang.strip()
        if not language:
            continue
        values = as_non_empty_str_list(raw_values)
        if values:
            out[language] = values
    if not out:
        fail("titles must contain at least one non-empty language/title entry")
    return out


def normalize_scrapers(raw_scrapers: object) -> list[dict[str, object]]:
    if raw_scrapers is None:
        return []
    if not isinstance(raw_scrapers, list):
        fail("scraper must be a list")
    out: list[dict[str, object]] = []
    for raw in raw_scrapers:
        if not isinstance(raw, dict):
            fail("scraper entry must be an object")
        channel = str(raw.get("channel", "")).strip()
        url = str(raw.get("url", "")).strip()
        if not channel or not url:
            continue
        out.append(
            {"channel": channel, "url": url, "enabled": bool(raw.get("enabled", True))}
        )
    return out


def normalize_volumes(raw_volumes: object) -> list[dict[str, object]]:
    if raw_volumes is None:
        return []
    if not isinstance(raw_volumes, list):
        fail("volumes must be a list")
    out: list[dict[str, object]] = []
    for raw in raw_volumes:
        if not isinstance(raw, dict):
            fail("volume entry must be an object")
        if "start" not in raw:
            fail("volume entry must contain 'start'")
        start = float(raw["start"])
        end = raw.get("end")
        if end is not None:
            end = float(end)
        title = raw.get("title")
        if title is not None:
            title = str(title).strip() or None
        out.append({"title": title, "start": start, "end": end})
    return out


def build_metadata_and_images(
    manifest: dict[str, object], root: pathlib.Path
) -> tuple[dict[str, object], list[pathlib.Path]]:
    metadata: dict[str, object] = {
        "titles": normalize_titles(manifest.get("titles")),
        "kind": str(manifest.get("kind", "manga")).strip() or "manga",
        "description": (
            str(manifest["description"]).strip() if manifest.get("description") else None
        ),
        "tags": [parse_tag_item(item) for item in (manifest.get("tags") or [])],
        "status": int(manifest.get("status", 0)),
        "visibility": int(manifest.get("visibility", 0)),
        "uploader": str(manifest.get("uploader", "unknown")).strip() or "unknown",
        "artists": as_non_empty_str_list(manifest.get("artists")),
        "authors": as_non_empty_str_list(manifest.get("authors")),
        "publishers": as_non_empty_str_list(manifest.get("publishers")),
        "sources": as_non_empty_str_list(manifest.get("sources")),
        "scraper": normalize_scrapers(manifest.get("scraper")),
        "volumes": normalize_volumes(manifest.get("volumes")),
        "cover_image_indexes": [],
        "art_image_indexes": [],
        "chapters": [],
    }

    image_paths: list[pathlib.Path] = []
    cover_paths = collect_paths(root, manifest.get("covers"))
    art_paths = collect_paths(root, manifest.get("art") or manifest.get("arts"))

    for path in cover_paths:
        metadata["cover_image_indexes"].append(len(image_paths))  # type: ignore[index]
        image_paths.append(path)
    for path in art_paths:
        metadata["art_image_indexes"].append(len(image_paths))  # type: ignore[index]
        image_paths.append(path)

    raw_chapters = manifest.get("chapters")
    if not isinstance(raw_chapters, list) or not raw_chapters:
        fail("manifest must contain a non-empty 'chapters' list")

    for raw_chapter in raw_chapters:
        if not isinstance(raw_chapter, dict):
            fail("chapter entry must be an object")
        chapter_value = raw_chapter.get("chapter")
        if chapter_value is None:
            fail("chapter entry is missing 'chapter'")

        chapter_meta: dict[str, object] = {
            "titles": as_non_empty_str_list(raw_chapter.get("titles")) or ["Untitled Chapter"],
            "chapter": float(chapter_value),
            "tags": [parse_tag_item(item) for item in (raw_chapter.get("tags") or [])],
            "sources": as_non_empty_str_list(raw_chapter.get("sources")),
            "release_date": (
                str(raw_chapter["release_date"]).strip()
                if raw_chapter.get("release_date")
                else None
            ),
            "versions": [],
        }

        raw_versions = raw_chapter.get("versions")
        if not isinstance(raw_versions, list) or not raw_versions:
            fail("chapter entry must contain a non-empty 'versions' list")

        for raw_version in raw_versions:
            if not isinstance(raw_version, dict):
                fail("chapter version entry must be an object")
            version_name = str(raw_version.get("version", "")).strip()
            if not version_name:
                fail("chapter version is missing 'version'")

            version_paths: list[pathlib.Path] = []
            if "images" in raw_version:
                version_paths = collect_paths(root, raw_version.get("images"))
            elif "path" in raw_version:
                version_paths = collect_paths(root, raw_version.get("path"))
            else:
                fail("chapter version requires either 'images' or 'path'")

            image_indexes = []
            for image_path in version_paths:
                image_indexes.append(len(image_paths))
                image_paths.append(image_path)

            chapter_meta["versions"].append(
                {
                    "version": version_name,
                    "image_indexes": image_indexes,
                    "link": (
                        str(raw_version["link"]).strip()
                        if raw_version.get("link")
                        else None
                    ),
                }
            )  # type: ignore[index]

        metadata["chapters"].append(chapter_meta)  # type: ignore[index]

    if not image_paths:
        fail("manifest does not resolve to any images")
    return metadata, image_paths


def write_bundle(out_path: pathlib.Path, metadata_bytes: bytes, images: list[pathlib.Path]) -> None:
    if len(metadata_bytes) > MAX_U32:
        fail("metadata payload exceeds 4 GiB limit")
    if len(images) > MAX_U32:
        fail("image count exceeds 32-bit limit")

    payload = bytearray()
    payload.extend(MANGA_MAGIC)
    payload.extend(struct.pack("<I", len(metadata_bytes)))
    payload.extend(metadata_bytes)
    payload.extend(struct.pack("<I", len(images)))

    for image in images:
        data = image.read_bytes()
        if len(data) > MAX_U32:
            fail(f"image exceeds 4 GiB limit: {image}")
        payload.extend(struct.pack("<I", len(data)))
        payload.extend(data)

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_bytes(payload)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Create an .mrmang bundle from a folder + JSON manifest."
    )
    parser.add_argument(
        "folder",
        type=pathlib.Path,
        help="Root folder that contains the manifest and chapter assets",
    )
    parser.add_argument(
        "--json",
        dest="manifest_name",
        default="export.json",
        help="Manifest file name/path (default: export.json)",
    )
    parser.add_argument(
        "--out",
        default="bundle.mrmang",
        help="Output file path (default: bundle.mrmang inside folder)",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    root = args.folder.resolve()
    if not root.exists() or not root.is_dir():
        fail(f"folder does not exist or is not a directory: {root}")

    manifest_path = resolve_path(root, args.manifest_name)
    if not manifest_path.exists():
        fail(f"manifest not found: {manifest_path}")

    out_path = resolve_path(root, args.out)
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if not isinstance(manifest, dict):
        fail("manifest root must be a JSON object")

    metadata, image_paths = build_metadata_and_images(manifest, root)
    metadata_bytes = build_metadata_bytes(metadata)
    write_bundle(out_path, metadata_bytes, image_paths)

    print(f"wrote: {out_path}")
    print(f"images: {len(image_paths)}")
    print(f"chapters: {len(metadata['chapters'])}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
