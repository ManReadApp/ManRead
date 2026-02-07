# Storage Crate: Dev Docs

## TL;DR

Flow is always:

1. `register_temp_file` (enqueue processing/upload)
2. `take` (wait for uploaded handle)
3. builder `build(...)` (final path rename)

This keeps request code small and backend-agnostic.

## Design Philosophy

- One public pipeline, many backends.
- Heavy work belongs in workers, not call sites.
- Upload first to temp key, finalize later with `rename`.
- Keep path building semantic (typed builders), not stringly in app code.

## Where Things Live

- `crates/storage/src/lib.rs`
  - `StorageSystem`
  - worker orchestration
  - in-memory state machine for registered files
- `crates/storage/src/backends/mod.rs`
  - `StorageReader` and `StorageWriter` traits
  - stream/object/options shared types
- `crates/storage/src/builders.rs`
  - `FileBuilder` and typed wrappers (`Cover...`, `UserCover...`, `MangaPage...`)
- `crates/storage/src/error.rs`
  - typed storage + processing errors

## Worker Model

`register_temp_file`:

- detects special containers by magic header
- detects type/ext
- inserts entry as `Processing`
- runs async worker (limited by semaphore)
- worker does:
  - image conversion only when needed (unsupported image -> JPEG)
  - image dimensions extraction
  - backend upload (`write`)
- marks entry:
  - `Uploaded { handle }` on success
  - `Failed { error }` on failure

`take` blocks on the entry notify until it becomes uploaded or failed.

## Container Types

`register_temp_file` now returns `RegisterTempResult`:

- `File(FileId)` for regular uploads
- `Chapter(Vec<FileId>)` for chapter containers or PDFs
- `Manga(RegisteredMangaTemp)` for manga containers

Magic headers:

- chapter: `MRCHAP01`
- manga: `MRMANG01`

Chapter container payload: image blobs only.
Manga container payload: bincode metadata + ordered image blobs.

`RegisteredMangaTemp.chapter_image_indexes` preserves index mapping from metadata.

### PDF Input

If input is PDF (`application/pdf`), pages are split to PNG files and treated like a chapter (`Vec<FileId>`).
Current implementation uses `pdftoppm` in a worker.

## Internal State

Each registered file is tracked in an in-memory map:

- metadata: ext + optional dims
- state:
  - `Processing { notify }`
  - `Uploaded { handle }`
  - `Failed { error }`

State transitions are explicit helper methods on `StoredFile`.

## Storage Traits

`StorageWriter`:

- `write(key, options, stream)`
- `rename(orig_key, target_key)`

`StorageReader`:

- `get(key, options)`

Built-in backends:

- `MemStorage` (`memory.rs`)
- `DelayStorage` (`delay.rs`)
- `EncryptedStorage` (`aes_gcm.rs`)
- `DiskStorage` (`disk.rs`, feature-gated)

## Builders (Fancy Path Builders)

`take` returns `FileBuilder` with temp uploaded handle.

Wrappers add semantic path prefixes:

- `CoverFileBuilder` -> `covers/...`
- `UserCoverFileBuilder` -> `users/icon/...`
- `MangaPageFileBuilder` -> `mangas/{manga}/{chapter}/{version}/...`

`build(id)` sets extension and calls backend `rename` to finalize object location.

## Performance Notes

- Allowed image formats avoid full decode for dimensions:
  - header-based dimension probing (`image_dimensions` path)
  - upload is streamed from file
- Unsupported image formats are fully decoded and re-encoded to JPEG in worker.
- Semaphore bounds concurrent heavy processing (`transcode_sem`).

### Disk Backend Copy Behavior

For `DiskStorage`, there is an extra copy in the pipeline:

- input starts in local temp file (from upload/request side)
- then streams into backend temp file (`DiskStorage::write`)
- then backend temp file is renamed to final backend location

So local-temp -> backend-temp is a redundant copy, but in this architecture it is expected and effectively unavoidable because the backend write boundary is stream-based and backend ownership is finalized by its own atomic move/rename step.

## Encoding Script

See:

- `crates/storage/scripts/encode_bundles.py`

Usage:

- `python3 scripts/encode_bundles.py chapter --out chapter.bin img1.png img2.png`
- `python3 scripts/encode_bundles.py manga --meta-bin metadata.bincode --out manga.bin img1.png img2.png`
