export function getCoverUrl(manga_id: string, ext: string) {
    return `/api/v1/image-no-auth/cover/${manga_id}.${ext === 'qoi' ? 'avif' : ext}`;
}