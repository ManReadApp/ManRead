use export::manga::StringList;

fn aa(ma: db::manga::Manga) {
    export::manga::Manga {
        titles: ma
            .titles
            .into_iter()
            .map(|v| (v.0, StringList { items: v.1 }))
            .collect(),
        kind: String::new(),
        description: None,
        tags: Vec::new(),
        status: 0,
        visibility: 0,
        uploader: String::new(),
        artists: Vec::new(),
        authors: Vec::new(),
        covers: Vec::new(),
        chapters: Vec::new(),
        sources: Vec::new(),
        scraper: Vec::new(),
        updated: None,
        created: None,
        art_ext: Vec::new(),
        publishers: Vec::new(),
        volumes: Vec::new(),
    };
}
