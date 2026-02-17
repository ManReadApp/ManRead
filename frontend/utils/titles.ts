//TODO: global
const prio = ["en"];
type TitleEntry = string[] | { items: string[] };
type TitleMap = { [key: string]: TitleEntry };

const normalizeItems = (entry?: TitleEntry) => {
    if (!entry) return [];
    return Array.isArray(entry) ? entry : entry.items ?? [];
};

export function getTitle(titles: TitleMap) {
    for (const pr of prio) {
        const title = normalizeItems(titles[pr]);
        if (title && title.length > 0) {
            return title[0];
        }
    }
    for (const entry of Object.values(titles)) {
        const first = normalizeItems(entry)[0];
        if (first) return first;
    }

    return "";
}

export function getOtherTitles(titles: TitleMap, main_title: string) {
    const builder: string[] = [];
    for (const title_key in titles) {
        const title_array = normalizeItems(titles[title_key]);
        for (const title of title_array) {
            if (main_title !== title) {
                builder.push(title);
            }
        }
    }
    if (builder.length > 0) {
        return builder.join(", ");
    } else {
        return null;
    }
}
