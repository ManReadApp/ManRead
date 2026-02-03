import {decode} from '@jsquash/qoi';

export type Ret = DataObject | {
    offset: number,
    message: "Unreachable" | "Unreachable2" | "Loading"
};

export function isDataObject(obj: any): obj is DataObject {
    return obj && typeof obj === "object" && ("chapter_id" in obj);
}

export interface DataObject {
    chapter_id: string;
    offset: number;
    complete?: {
        version_id: string;
        page: Page;
    }
}

export interface Page {
    ext: string;
    height: number;
    id: string;
    width: number;
    page: number;
}

export type Cache = Record<string, Ref<{ success?: string; error?: unknown }>>;
export type ImageItem = {
    manga_id: string;
    chapter_id: string;
    version_id: string;
    page: number;
    file_ext: string;
};

type ImageRef = Ref<{ success?: string; error?: unknown }>;


let lock = false;

export async function getImages(cache: Cache, new_items: ImageItem[]) {
    if (lock) return;
    lock = true;
    const keep = new Set<string>()
    const add: [string, Ref][] = []
    for (const item of new_items) {
        const key: string = item.chapter_id + item.version_id + item.page;
        const img = cache[key]
        if (img) {
            keep.add(key);
        } else {
            const r = ref<{ success?: string; error?: unknown }>({})
            const _ = downloadImageWithRef(r, item.manga_id, item.chapter_id, item.version_id, item.page, item.file_ext)
            add.push([key, r])
        }
    }
    for (const item in cache) {
        if (!keep.has(item)) {
            console.log("delete", item);
            const a = cache[item];
            if (a.value.success) {
                URL.revokeObjectURL(a.value.success)
            }
            delete cache[item]
        }
    }
    for (const [key, item] of add) {
        cache[key] = item
    }
    lock = false;

}

async function downloadImageWithRef(ref: ImageRef, manga_id: string, chapter_id: string, version_id: string, page: number, file_ext: string) {
    try {
        ref.value = {success: await downloadImage(manga_id, chapter_id, version_id, page, file_ext)}
    } catch (e) {
        ref.value = {error: e};
    }
}

async function downloadImage(manga_id: string, chapter_id: string, version_id: string, page: number, file_ext: string): Promise<string> {
    const blob: Blob = await $fetch('http://127.0.0.1:8082/api/v1/image/page', {
        method: 'POST', body: {
            chapter_id: chapter_id, file_ext: file_ext, manga_id: manga_id, page: page, version_id: version_id
        },
        headers: {Authorization: `Bearer ${await getAccessToken()}`}
    })
    if (file_ext == 'qoi') {
        const arrayBuffer = await blob.arrayBuffer();
        const qoiImage = await decode(arrayBuffer);
        const canvas = document.createElement("canvas");
        const ctx = canvas.getContext("2d");

        if (ctx) {
            canvas.width = qoiImage.width;
            canvas.height = qoiImage.height;
            ctx.putImageData(qoiImage, 0, 0);
        }

        return await new Promise<string>((resolve, reject) => {
            canvas.toBlob((blob) => {
                if (!blob) {
                    reject(new Error("Failed to get blob"));
                    return;
                }
                resolve(URL.createObjectURL(blob));
            });
        })
    } else {
        return URL.createObjectURL(blob);
    }
}