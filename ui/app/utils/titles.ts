//TODO: global
const prio = ["en"];
export function getTitle(titles: { [key: string]: string[] }) {
  for (const pr of prio) {
    const title = titles[pr];
    if (title && title.length > 0) {
      return title[0] ?? "";
    }
  }
  return Object.entries(titles)[0]?.[1][0] ?? "No Title";
}

export function getOtherTitles(
  titles: { [key: string]: string[] },
  main_title: string,
) {
  const builder: string[] = [];
  for (const title_key in titles) {
    const title_array = titles[title_key] ?? [];
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
