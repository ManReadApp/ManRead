use std::{collections::HashMap, path::PathBuf, sync::Arc};

use api_structure::{
    now,
    req::LoginRequest,
    search::{Array, Item, ItemData, ItemOrArray, ItemValue, SearchRequest as MangaSearchRequest},
    v1::{
        self, ActivationTokenKind, AddMangaRequest, Claim, EditChapterRequest, EditMangaRequest,
        Gender, LoginWithEmailAndPassword, LoginWithUsernameAndPassword, PaginationRequest,
        PasswordChange, ResetPasswordRequest, Role, SearchRequest as SimpleSearchRequest, Status,
        StringList, Tag, TagList, TagSex, UpdateUserRequest,
    },
};
use chrono::Utc;
use db::{init_db, DbConfig, DbHandle, MemoryDbConfig};
use futures_util::StreamExt as _;
use serde::Deserialize;
use std::time::Duration;
use storage::{FileId, MemStorage, RegisterTempResult, StorageSystem};
use tokio::io::AsyncWriteExt as _;

use crate::{
    actions::{
        auth::AuthAction,
        chapter::ChapterActions,
        chapter_version::ChapterVersionActions,
        crytpo::CryptoService,
        kind::KindActions,
        lists::ListActions,
        manga::{MangaActions, VolumeRange},
        reader::ReaderActions,
        tags::TagActions,
        token::TokenAction,
        user::UserActions,
    },
    error::ApiError,
};

const PNG_1X1: &[u8] = b"\x89PNG\r\n\x1a\n\
\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15\xc4\x89\
\x00\x00\x00\x0cIDAT\x08\xd7c\xf8\x0f\x00\x01\x01\x01\x00\x18\xdd\x8d\xb4\
\x00\x00\x00\x00IEND\xaeB`\x82";

struct RegisteredUser {
    id: String,
    claim: Claim,
}

struct CreatedChapter {
    chapter_id: String,
    version_id: String,
    chapter_version_id: String,
}

struct TestCtx {
    db: DbHandle,
    crypto: Arc<CryptoService>,
    auth: AuthAction,
    chapter: ChapterActions,
    chapter_version: ChapterVersionActions,
    kind: KindActions,
    list: ListActions,
    manga: MangaActions,
    reader: ReaderActions,
    storage: Arc<StorageSystem>,
    tag: TagActions,
    token: TokenAction,
    user: UserActions,
}

#[derive(Deserialize)]
struct TokenRow {
    token: String,
}

fn list(items: &[&str]) -> StringList {
    StringList {
        items: items.iter().map(|v| (*v).to_owned()).collect(),
    }
}

fn names(name: &str) -> HashMap<String, StringList> {
    HashMap::from([("en".to_owned(), list(&[name]))])
}

fn tag(name: &str) -> Tag {
    Tag {
        tag: name.to_owned(),
        description: None,
        sex: TagSex::None,
    }
}

fn search_all() -> MangaSearchRequest {
    MangaSearchRequest {
        order: "created".to_owned(),
        desc: true,
        limit: 25,
        page: 1,
        query: Array {
            or: false,
            not: false,
            or_post: None,
            items: vec![],
        },
    }
}

fn search_by_title(title: &str) -> MangaSearchRequest {
    MangaSearchRequest {
        order: "created".to_owned(),
        desc: true,
        limit: 25,
        page: 1,
        query: Array {
            or: false,
            not: false,
            or_post: None,
            items: vec![ItemOrArray::Item(Item::new(ItemData {
                name: "title".to_owned(),
                value: ItemValue::String(title.to_owned()),
            }))],
        },
    }
}

impl TestCtx {
    async fn new() -> Self {
        let namespace = format!("ns_{}", helper::random_string(10));
        let database = format!("db_{}", helper::random_string(10));
        let db = init_db(DbConfig::Memory(MemoryDbConfig {
            namespace,
            database,
        }))
        .await
        .expect("memory db should initialize");

        let root = std::env::temp_dir().join(format!("apiv2-tests-{}", helper::random_string(8)));
        tokio::fs::create_dir_all(&root)
            .await
            .expect("test temp root should be created");

        let storage = Arc::new(
            StorageSystem::new(PathBuf::as_path(&root), Arc::new(MemStorage::new()))
                .await
                .expect("memory storage should initialize"),
        );
        let crypto = Arc::new(CryptoService::new(b"unit-test-secret".to_vec()));

        let auth = AuthAction {
            users: db.users.clone(),
            crypto: crypto.clone(),
            token: db.tokens.clone(),
            fs: storage.clone(),
        };
        let chapter = ChapterActions {
            chapters: db.chapters.clone(),
            tags: db.tags.clone(),
            versions: db.versions.clone(),
            chapter_versions: db.chapter_versions.clone(),
            mangas: db.mangas.clone(),
            pages: db.pages.clone(),
            fs: storage.clone(),
        };
        let chapter_version = ChapterVersionActions {
            versions: db.versions.clone(),
            chapters: db.chapters.clone(),
            chapter_versions: db.chapter_versions.clone(),
            pages: db.pages.clone(),
            fs: storage.clone(),
        };
        let kind = KindActions {
            kinds: db.kinds.clone(),
        };
        let list = ListActions {
            mangas: db.mangas.clone(),
            lists: db.lists.clone(),
        };
        let manga = MangaActions {
            mangas: db.mangas.clone(),
            chapters: db.chapters.clone(),
            tags: db.tags.clone(),
            kinds: db.kinds.clone(),
            users: db.users.clone(),
            lists: db.lists.clone(),
            versions: db.versions.clone(),
            chapter_versions: db.chapter_versions.clone(),
            pages: db.pages.clone(),
            fs: storage.clone(),
        };
        let reader = ReaderActions {
            progresses: db.progress.clone(),
            chapters: db.chapters.clone(),
            pages: db.pages.clone(),
            chapter_versions: db.chapter_versions.clone(),
            mangas: db.mangas.clone(),
            lists: db.lists.clone(),
            kinds: db.kinds.clone(),
        };
        let tag = TagActions {
            tags: db.tags.clone(),
        };
        let token = TokenAction {
            token: db.tokens.clone(),
        };
        let user = UserActions {
            users: db.users.clone(),
            crypto: crypto.clone(),
            fs: storage.clone(),
            tags: db.tags.clone(),
        };

        Self {
            db,
            crypto,
            auth,
            chapter,
            chapter_version,
            kind,
            list,
            manga,
            reader,
            storage,
            tag,
            token,
            user,
        }
    }

    async fn upload_png(&self) -> String {
        let mut tf = self
            .storage
            .new_temp_file()
            .await
            .expect("new temp file should work");
        tf.write_all(PNG_1X1)
            .await
            .expect("png bytes should be written");
        tf.flush().await.expect("temp file should be flushed");

        let id = self
            .storage
            .register_temp_file(tf)
            .await
            .expect("temp registration should succeed");

        match id {
            RegisterTempResult::File(file_id) => file_id.inner(),
            RegisterTempResult::Chapter(_) => panic!("expected a file handle"),
            RegisterTempResult::Manga(_) => panic!("expected a file handle"),
        }
    }

    async fn register_user(&self, name: &str, email: &str, password: &str) -> RegisteredUser {
        let icon = self.upload_png().await;
        let jwt = self
            .auth
            .register(
                &email.to_owned(),
                name.to_owned(),
                &password.to_owned(),
                Gender::Unknown,
                Utc::now(),
                Some(FileId::new(icon)),
            )
            .await
            .expect("user registration should succeed");

        let claim = self
            .crypto
            .get_claim(&jwt.access_token)
            .expect("access token should be decodable");

        RegisteredUser {
            id: claim.id.clone(),
            claim,
        }
    }

    async fn latest_token_by_kind(&self, kind: ActivationTokenKind) -> String {
        let mut result = self
            .db
            .session
            .query(
                "SELECT token, created FROM auth_tokens WHERE kind = $kind ORDER BY created DESC LIMIT 1",
            )
            .bind(("kind", u32::from(kind)))
            .await
            .expect("token query should succeed");

        let mut rows: Vec<TokenRow> = result.take(0).expect("token rows should deserialize");
        rows.remove(0).token
    }

    async fn create_manga(&self, uid: &str, title: &str, kind: &str) -> String {
        let image_temp_name = self.upload_png().await;
        let request = AddMangaRequest {
            names: names(title),
            kind: kind.to_owned(),
            status: Status::Ongoing,
            description: Some(format!("description for {title}")),
            tags: vec![tag("action"), tag("adventure")],
            image_temp_name,
            authors: vec!["author-a".to_owned()],
            publishers: vec!["publisher-a".to_owned()],
            artists: vec!["artist-a".to_owned()],
            sources: vec!["https://source.example".to_owned()],
            scrapers: vec![v1::Scraper {
                channel: "en".to_owned(),
                url: "https://scraper.example/feed".to_owned(),
            }],
        };

        self.manga
            .create(request, uid)
            .await
            .expect("manga creation should succeed");

        let (results, _) = self
            .manga
            .search(search_by_title(title), uid)
            .await
            .expect("manga should be searchable");

        results
            .into_iter()
            .find(|item| {
                item.titles
                    .values()
                    .flatten()
                    .any(|candidate| candidate == title)
            })
            .expect("created manga should be present in search results")
            .manga_id
    }

    async fn create_chapter(
        &self,
        manga_id: &str,
        episode: f64,
        version_name: &str,
        image_count: usize,
    ) -> CreatedChapter {
        let mut images = Vec::new();
        for _ in 0..image_count {
            images.push(self.upload_png().await);
        }

        self.chapter
            .add(
                manga_id,
                vec![format!("chapter {episode}")],
                episode,
                version_name,
                images,
                vec![],
                vec!["https://chapter.example".to_owned()],
                Some(Utc::now()),
            )
            .await
            .expect("chapter add should succeed");

        let manga = self
            .db
            .mangas
            .get(manga_id)
            .await
            .expect("manga should exist");
        let mut chapters = self
            .db
            .chapters
            .get_detail(manga.chapters.into_iter())
            .await
            .expect("manga chapters should load");
        let version_id = self
            .db
            .versions
            .get(version_name)
            .await
            .expect("version should exist")
            .id()
            .to_string();
        let chapter = chapters
            .drain(..)
            .find(|entry| {
                entry.data.chapter == episode
                    && entry
                        .data
                        .versions
                        .keys()
                        .any(|key| key.ends_with(&version_id))
            })
            .expect("chapter should contain requested version");

        let chapter_version_id = chapter
            .data
            .versions
            .iter()
            .find(|(key, _)| key.ends_with(&version_id))
            .expect("chapter should contain requested version")
            .1
            .id()
            .to_string();

        CreatedChapter {
            chapter_id: chapter.id.id().to_string(),
            version_id,
            chapter_version_id,
        }
    }
}

#[actix_web::test]
async fn auth_and_token_actions_cover_primary_flows() {
    let ctx = TestCtx::new().await;
    let user = ctx
        .register_user("alice", "alice@example.com", "password-1")
        .await;

    let by_mail = ctx
        .auth
        .get_user_id(true, &"alice@example.com".to_owned())
        .await
        .expect("lookup by mail should succeed");
    let by_name = ctx
        .auth
        .get_user_id(false, &"alice".to_owned())
        .await
        .expect("lookup by name should succeed");
    assert_eq!(by_mail.id.id(), by_name.id.id());

    ctx.auth
        .login(LoginRequest::Username(LoginWithUsernameAndPassword {
            username: "alice".to_owned(),
            password: "password-1".to_owned(),
        }))
        .await
        .expect("username login should succeed");
    ctx.auth
        .login(LoginRequest::Email(LoginWithEmailAndPassword {
            email: "alice@example.com".to_owned(),
            password: "password-1".to_owned(),
        }))
        .await
        .expect("email login should succeed");
    assert!(matches!(
        ctx.auth
            .login(LoginRequest::Email(LoginWithEmailAndPassword {
                email: "alice@example.com".to_owned(),
                password: "wrong-password".to_owned(),
            }))
            .await,
        Err(ApiError::PasswordIncorrect)
    ));

    let issued = ctx
        .auth
        .login(LoginRequest::Username(LoginWithUsernameAndPassword {
            username: "alice".to_owned(),
            password: "password-1".to_owned(),
        }))
        .await
        .expect("login should produce refresh token");
    ctx.auth
        .refresh(&issued.refresh_token)
        .await
        .expect("refresh before logout should succeed");
    tokio::time::sleep(Duration::from_millis(1100)).await;
    ctx.auth
        .logout(&user.claim)
        .await
        .expect("logout should persist generated timestamp");
    assert!(matches!(
        ctx.auth.refresh(&issued.refresh_token).await,
        Err(ApiError::ExpiredToken)
    ));

    ctx.auth
        .request_reset_password(user.id.clone())
        .await
        .expect("request reset should create token");
    let reset_key = ctx
        .latest_token_by_kind(ActivationTokenKind {
            single: true,
            kind: Role::NotVerified,
        })
        .await;
    ctx.auth
        .reset_password(ResetPasswordRequest {
            ident: "alice".to_owned(),
            email: false,
            key: reset_key,
            password: "password-2".to_owned(),
        })
        .await
        .expect("reset password should succeed");

    assert!(matches!(
        ctx.auth
            .login(LoginRequest::Username(LoginWithUsernameAndPassword {
                username: "alice".to_owned(),
                password: "password-1".to_owned(),
            }))
            .await,
        Err(ApiError::PasswordIncorrect)
    ));
    ctx.auth
        .login(LoginRequest::Username(LoginWithUsernameAndPassword {
            username: "alice".to_owned(),
            password: "password-2".to_owned(),
        }))
        .await
        .expect("new password should be active");

    let verify_kind = ActivationTokenKind {
        single: true,
        kind: Role::User,
    };
    ctx.token
        .create_token(Some(user.id.clone()), verify_kind.clone())
        .await
        .expect("admin token creation should work");

    let listed = ctx
        .token
        .list_tokens(1, 50)
        .await
        .expect("token list should succeed");
    assert!(!listed.is_empty());

    let verify_key = ctx.latest_token_by_kind(verify_kind).await;
    let verified = ctx
        .auth
        .verify(&verify_key, &user.claim)
        .await
        .expect("verification token should update role");
    let verified_claim = ctx
        .crypto
        .get_claim(&verified.access_token)
        .expect("verification jwt should decode");
    assert_eq!(verified_claim.role, Role::User);

    let token_id = listed[0].id.id().to_string();
    ctx.token
        .delete_token(&token_id)
        .await
        .expect("token delete should succeed");
}

#[actix_web::test]
async fn auth_register_rejects_case_only_duplicate_identity() {
    let ctx = TestCtx::new().await;
    let icon_a = ctx.upload_png().await;
    ctx.auth
        .register(
            &"alice@example.com".to_owned(),
            "alice".to_owned(),
            &"password-1".to_owned(),
            Gender::Unknown,
            Utc::now(),
            Some(FileId::new(icon_a)),
        )
        .await
        .expect("first registration should succeed");

    let icon_b = ctx.upload_png().await;
    assert!(matches!(
        ctx.auth
            .register(
                &"ALICE@EXAMPLE.COM".to_owned(),
                "alice-2".to_owned(),
                &"password-2".to_owned(),
                Gender::Unknown,
                Utc::now(),
                Some(FileId::new(icon_b)),
            )
            .await,
        Err(ApiError::EmailExists)
    ));

    let icon_c = ctx.upload_png().await;
    assert!(matches!(
        ctx.auth
            .register(
                &"alice-3@example.com".to_owned(),
                "ALICE".to_owned(),
                &"password-3".to_owned(),
                Gender::Unknown,
                Utc::now(),
                Some(FileId::new(icon_c)),
            )
            .await,
        Err(ApiError::NameExists)
    ));
}

#[actix_web::test]
async fn user_actions_edit_info_list_and_search_work() {
    let ctx = TestCtx::new().await;
    let user = ctx
        .register_user("bob", "bob@example.com", "password-a")
        .await;

    let icon_temp = ctx.upload_png().await;
    let thumb_temp = ctx.upload_png().await;
    ctx.user
        .edit(
            UpdateUserRequest {
                name: Some(list(&["bob", "bobby"])),
                password: Some(PasswordChange {
                    old_password: "password-a".to_owned(),
                    new_password: "password-b".to_owned(),
                }),
                icon_temp_name: Some(icon_temp),
                description: Some("new profile text".to_owned()),
                links: Some(list(&["https://example.com/u/bob"])),
                thumbnail: Some(thumb_temp),
            },
            &user.claim,
        )
        .await
        .expect("user edit should succeed");

    let info = ctx
        .user
        .info(&user.id)
        .await
        .expect("user info should be readable");
    assert!(info.names.contains(&"bob".to_owned()));
    assert_eq!(info.bio.as_deref(), Some("new profile text"));
    assert_eq!(info.links, vec!["https://example.com/u/bob".to_owned()]);
    assert!(info.icon_ext.is_some());
    assert!(info.thumb_ext.is_some());

    let list = ctx
        .user
        .list(PaginationRequest { page: 1, limit: 10 })
        .await
        .expect("user list should succeed");
    assert!(list.iter().any(|entry| entry.id == user.id));

    let search = ctx
        .user
        .search(SimpleSearchRequest {
            query: "bo".to_owned(),
            page: 1,
            limit: 10,
        })
        .await
        .expect("user search should succeed");
    assert!(search.iter().any(|entry| entry.id == user.id));

    assert!(matches!(
        ctx.user
            .edit(
                UpdateUserRequest {
                    name: None,
                    password: Some(PasswordChange {
                        old_password: "wrong-password".to_owned(),
                        new_password: "x".to_owned(),
                    }),
                    icon_temp_name: None,
                    description: None,
                    links: None,
                    thumbnail: None,
                },
                &user.claim,
            )
            .await,
        Err(ApiError::PasswordIncorrect)
    ));
}

#[actix_web::test]
async fn user_delete_sets_disabled_flag_without_removing_record() {
    let ctx = TestCtx::new().await;
    let user = ctx
        .register_user("todo", "todo@example.com", "password")
        .await;
    ctx.user
        .delete(&user.id)
        .await
        .expect("user delete should soft-delete");

    let user_row = ctx
        .db
        .users
        .info(&user.id)
        .await
        .expect("user row should still exist");
    assert!(user_row.data.disabled);

    assert!(matches!(
        ctx.auth
            .login(LoginRequest::Username(LoginWithUsernameAndPassword {
                username: "todo".to_owned(),
                password: "password".to_owned(),
            }))
            .await,
        Err(ApiError::NotFoundInDB)
    ));
}

#[actix_web::test]
async fn manga_actions_cover_create_search_edit_and_state_transitions() {
    let ctx = TestCtx::new().await;
    let user = ctx
        .register_user("creator", "creator@example.com", "password")
        .await;
    let manga_id = ctx.create_manga(&user.id, "Skyline", "manhwa").await;

    let info = ctx
        .manga
        .info(manga_id.clone(), &user.id)
        .await
        .expect("manga info should succeed");
    assert_eq!(info.kind, "manhwa");
    assert!(info
        .titles
        .get("en")
        .expect("english titles should exist")
        .items
        .contains(&"Skyline".to_owned()));

    let extra_cover = ctx.upload_png().await;
    ctx.manga
        .add_cover(&manga_id, &extra_cover)
        .await
        .expect("add cover should succeed");
    ctx.manga
        .remove_cover(&manga_id, 1)
        .await
        .expect("remove cover should succeed");

    let extra_art = ctx.upload_png().await;
    ctx.manga
        .add_art(&manga_id, &extra_art)
        .await
        .expect("add art should succeed");
    ctx.manga
        .remove_art(&manga_id, 0)
        .await
        .expect("remove art should succeed");

    ctx.manga
        .edit(EditMangaRequest {
            manga_id: manga_id.clone(),
            names: names("Skyline Redux"),
            kind: "manga".to_owned(),
            status: Status::Completed,
            description: Some("edited".to_owned()),
            tags: vec![],
            authors: vec!["author-b".to_owned()],
            publishers: vec!["publisher-b".to_owned()],
            artists: vec!["artist-b".to_owned()],
            sources: vec!["https://new.source".to_owned()],
            scrapers: vec![v1::Scraper {
                channel: "en".to_owned(),
                url: "https://scraper.example/new".to_owned(),
            }],
        })
        .await
        .expect("manga edit should succeed");

    let after_edit = ctx
        .manga
        .info(manga_id.clone(), &user.id)
        .await
        .expect("edited manga should still load");
    assert_eq!(after_edit.kind, "manga");
    assert_eq!(after_edit.status, Status::Completed);
    assert!(after_edit
        .titles
        .get("en")
        .expect("english titles should exist")
        .items
        .contains(&"Skyline Redux".to_owned()));

    let (search, _) = ctx
        .manga
        .search(search_by_title("Redux"), &user.id)
        .await
        .expect("search should succeed");
    assert!(search.iter().any(|entry| entry.manga_id == manga_id));

    let home = ctx
        .manga
        .home(&user.id)
        .await
        .expect("home generation should succeed");
    assert!(!home.newest.is_empty());
    assert!(!home.random.is_empty());

    ctx.manga
        .set_volume_range(
            &manga_id,
            vec![
                VolumeRange {
                    start: 1.0,
                    end: Some(10.0),
                    title: Some("Volume 1".to_owned()),
                },
                VolumeRange {
                    start: 11.0,
                    end: None,
                    title: Some("Volume 2+".to_owned()),
                },
            ],
        )
        .await
        .expect("set volumes should succeed");

    let db_manga = ctx
        .db
        .mangas
        .get(&manga_id)
        .await
        .expect("db manga should load");
    assert_eq!(db_manga.volumes.len(), 2);

    ctx.manga
        .disable_scraper(&manga_id, "https://scraper.example/new")
        .await
        .expect("disable scraper should succeed");
    let disabled = ctx
        .db
        .mangas
        .get(&manga_id)
        .await
        .expect("manga should load");
    assert!(!disabled.scraper[0].enabled);
    ctx.manga
        .enable_scraper(&manga_id, "https://scraper.example/new")
        .await
        .expect("enable scraper should succeed");
    let enabled = ctx
        .db
        .mangas
        .get(&manga_id)
        .await
        .expect("manga should load");
    assert!(enabled.scraper[0].enabled);

    ctx.manga
        .delete(&manga_id)
        .await
        .expect("delete request should set review visibility");
    let review = ctx
        .manga
        .info(manga_id.clone(), &user.id)
        .await
        .expect("manga should still be readable");
    assert_eq!(review.visibility, v1::Visibility::AdminReview);

    ctx.manga
        .confirm_delete(&manga_id)
        .await
        .expect("confirm delete should hide manga");
    let hidden = ctx
        .manga
        .info(manga_id, &user.id)
        .await
        .expect("manga should still be readable");
    assert_eq!(hidden.visibility, v1::Visibility::Hidden);
}

#[actix_web::test]
async fn manga_export_emits_protobuf_metadata_and_indexed_images() {
    let ctx = TestCtx::new().await;
    let user = ctx
        .register_user("exporter", "exporter@example.com", "password")
        .await;
    let manga_id = ctx.create_manga(&user.id, "Export Manga", "manga").await;
    let chapter = ctx.create_chapter(&manga_id, 1.0, "en", 2).await;

    let mut payload = Vec::new();
    let mut stream = ctx
        .manga
        .export(&manga_id)
        .await
        .expect("manga export should succeed");
    while let Some(chunk) = stream.next().await {
        payload.extend_from_slice(&chunk.expect("export stream chunk should read"));
    }
    assert_eq!(&payload[..8], b"MRMANG01");

    let mut cursor = 8usize;
    let metadata_len = u32::from_le_bytes(
        payload[cursor..cursor + 4]
            .try_into()
            .expect("metadata length should exist"),
    ) as usize;
    cursor += 4;

    let metadata: export::manga::MangaBundleMetadata =
        export::try_from_bytes(&payload[cursor..cursor + metadata_len])
            .expect("metadata should decode");
    cursor += metadata_len;

    assert_eq!(metadata.manga_id, manga_id);
    assert_eq!(metadata.chapters.len(), 1);
    assert_eq!(metadata.chapters[0].chapter_id, chapter.chapter_id);
    assert_eq!(metadata.chapters[0].version_id, chapter.version_id);
    assert_eq!(metadata.chapters[0].image_indexes, vec![0, 1]);

    let image_count = u32::from_le_bytes(
        payload[cursor..cursor + 4]
            .try_into()
            .expect("image count should exist"),
    ) as usize;
    cursor += 4;
    assert_eq!(image_count, 2);

    for _ in 0..image_count {
        let image_len = u32::from_le_bytes(
            payload[cursor..cursor + 4]
                .try_into()
                .expect("image length should exist"),
        ) as usize;
        cursor += 4;
        let image_bytes = &payload[cursor..cursor + image_len];
        cursor += image_len;
        assert_eq!(
            infer::get(image_bytes).map(|kind| kind.mime_type()),
            Some("image/png")
        );
    }
    assert_eq!(cursor, payload.len());
}

#[actix_web::test]
async fn manga_relation_actions_are_symmetric() {
    let ctx = TestCtx::new().await;
    let user = ctx
        .register_user("rel", "rel@example.com", "password")
        .await;
    let first = ctx.create_manga(&user.id, "Relation A", "manga").await;
    let second = ctx.create_manga(&user.id, "Relation B", "manga").await;

    ctx.manga
        .add_relation(&first, &second)
        .await
        .expect("add relation should succeed");
    let first_info = ctx
        .manga
        .info(first.clone(), &user.id)
        .await
        .expect("first info should load");
    let second_info = ctx
        .manga
        .info(second.clone(), &user.id)
        .await
        .expect("second info should load");
    assert!(first_info
        .relations
        .iter()
        .any(|rel| rel.manga_id == second));
    assert!(second_info
        .relations
        .iter()
        .any(|rel| rel.manga_id == first));

    ctx.manga
        .remove_relation(&first, &second)
        .await
        .expect("remove relation should succeed");
    let first_after = ctx
        .manga
        .info(first, &user.id)
        .await
        .expect("first info should load");
    let second_after = ctx
        .manga
        .info(second, &user.id)
        .await
        .expect("second info should load");
    assert!(first_after.relations.is_empty());
    assert!(second_after.relations.is_empty());
}

#[actix_web::test]
async fn chapter_and_reader_actions_cover_add_edit_info_pages_progress_and_delete() {
    let ctx = TestCtx::new().await;
    let user = ctx
        .register_user("reader", "reader@example.com", "password")
        .await;
    let manga_id = ctx.create_manga(&user.id, "Reader Manga", "manga").await;

    let first = ctx.create_chapter(&manga_id, 1.0, "en", 2).await;
    let second = ctx.create_chapter(&manga_id, 2.0, "en", 1).await;

    let first_info = ctx
        .chapter
        .info(&first.chapter_id)
        .await
        .expect("chapter info should load");
    assert_eq!(first_info.chapter, 1.0);
    assert!(
        first_info
            .versions
            .iter()
            .any(|(key, value)| key.ends_with(&first.version_id)
                && value == &first.chapter_version_id)
    );

    assert!(matches!(
        ctx.chapter
            .add(
                &manga_id,
                vec!["duplicate".to_owned()],
                1.0,
                "en",
                vec![ctx.upload_png().await],
                vec![],
                vec![],
                None,
            )
            .await,
        Err(ApiError::ChapterVersionAlreadyExists)
    ));

    let duplicate_count = ctx
        .db
        .chapters
        .get_detail(
            ctx.db
                .mangas
                .get(&manga_id)
                .await
                .expect("manga should exist")
                .chapters
                .into_iter(),
        )
        .await
        .expect("chapter list should load")
        .into_iter()
        .filter(|entry| entry.data.chapter == 1.0)
        .count();
    assert_eq!(duplicate_count, 1);

    let edited_ts = (Utc::now().timestamp_millis() + 10_000) as u64;
    ctx.chapter
        .edit(EditChapterRequest {
            chapter_id: first.chapter_id.clone(),
            titles: Some(list(&["chapter 1 updated"])),
            chapter: Some(1.5),
            tags: Some(TagList { items: vec![] }),
            sources: Some(list(&["https://updated.chapter"])),
            release_date: Some(edited_ts),
            clear_release_date: false,
        })
        .await
        .expect("chapter edit should succeed");

    let edited = ctx
        .chapter
        .info(&first.chapter_id)
        .await
        .expect("edited chapter should load");
    assert_eq!(edited.chapter, 1.5);
    assert!(edited.release_date.is_some());

    ctx.chapter
        .edit(EditChapterRequest {
            chapter_id: first.chapter_id.clone(),
            titles: None,
            chapter: None,
            tags: None,
            sources: None,
            release_date: None,
            clear_release_date: true,
        })
        .await
        .expect("release date clear should succeed");
    let no_release = ctx
        .chapter
        .info(&first.chapter_id)
        .await
        .expect("chapter info should load");
    assert!(no_release.release_date.is_none());

    let before_progress = ctx
        .reader
        .info(&manga_id, None, &user.claim)
        .await
        .expect("reader info should load");
    assert_eq!(before_progress.open_chapter, first.chapter_id);

    ctx.reader
        .save_progress(1.2, &first.chapter_id, &user.claim)
        .await
        .expect("save progress should clamp and succeed");
    let after_progress = ctx
        .reader
        .info(&manga_id, None, &user.claim)
        .await
        .expect("reader info should load");
    assert_eq!(after_progress.open_chapter, second.chapter_id);
    assert_eq!(after_progress.progress, 0.0);

    let pages = ctx
        .reader
        .pages(&first.chapter_version_id)
        .await
        .expect("page info should load");
    assert_eq!(pages.pages.len(), 2);

    ctx.chapter
        .delete(&second.chapter_id)
        .await
        .expect("chapter delete should succeed");
    let manga_after = ctx
        .manga
        .info(manga_id, &user.id)
        .await
        .expect("manga info should load");
    assert!(!manga_after
        .chapters
        .iter()
        .any(|chapter| chapter.id == second.chapter_id));
}

#[actix_web::test]
async fn chapter_delete_removes_pages_and_storage_objects() {
    let ctx = TestCtx::new().await;
    let user = ctx
        .register_user("chapter-delete", "chapter-delete@example.com", "password")
        .await;
    let manga_id = ctx
        .create_manga(&user.id, "Delete Chapter Manga", "manga")
        .await;
    let chapter = ctx.create_chapter(&manga_id, 1.0, "en", 2).await;

    let chapter_version = ctx
        .db
        .chapter_versions
        .get(&chapter.chapter_version_id)
        .await
        .expect("chapter version should exist");
    let page_ids = chapter_version.pages.clone();
    let pages = ctx
        .db
        .pages
        .get(chapter_version.pages)
        .await
        .expect("page rows should exist");
    assert!(!pages.is_empty());
    for page in &pages {
        let key = format!(
            "mangas/{}/{}/{}/{}.{}",
            manga_id, chapter.chapter_id, chapter.version_id, page.data.page, page.data.ext
        );
        ctx.storage
            .reader
            .get(&key, &Default::default())
            .await
            .expect("chapter page should exist in storage");
    }

    ctx.chapter
        .delete(&chapter.chapter_id)
        .await
        .expect("chapter delete should succeed");

    assert!(matches!(
        ctx.reader.pages(&chapter.chapter_version_id).await,
        Err(ApiError::NotFoundInDB)
    ));
    let remaining_pages = ctx
        .db
        .pages
        .get(page_ids)
        .await
        .expect("page lookup should succeed");
    assert!(remaining_pages.is_empty());

    for page in pages {
        let key = format!(
            "mangas/{}/{}/{}/{}.{}",
            manga_id, chapter.chapter_id, chapter.version_id, page.data.page, page.data.ext
        );
        let err = match ctx.storage.reader.get(&key, &Default::default()).await {
            Ok(_) => panic!("chapter page should be deleted from storage"),
            Err(err) => err,
        };
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }
}

#[actix_web::test]
async fn chapter_version_actions_cover_edit_list_and_delete() {
    let ctx = TestCtx::new().await;
    let user = ctx
        .register_user("vuser", "vuser@example.com", "password")
        .await;
    let manga_id = ctx.create_manga(&user.id, "Versioned Manga", "manga").await;

    let _base = ctx.create_chapter(&manga_id, 1.0, "en", 1).await;
    let second_version = ctx.create_chapter(&manga_id, 1.0, "de", 1).await;

    ctx.chapter_version
        .edit(
            &second_version.version_id,
            Some("german".to_owned()),
            Some("translate aggressively".to_owned()),
        )
        .await
        .expect("chapter version edit should succeed");

    let list = ctx
        .chapter_version
        .list(PaginationRequest { page: 1, limit: 20 })
        .await
        .expect("chapter version list should succeed");
    assert!(list.iter().any(|item| item.name == "german"
        && item.translate_opts.as_deref() == Some("translate aggressively")));

    let chapter = ctx
        .chapter
        .info(&second_version.chapter_id)
        .await
        .expect("chapter info should load");
    let german_connection = chapter
        .versions
        .iter()
        .find(|(key, _)| key.ends_with(&second_version.version_id))
        .expect("german version should be present")
        .1
        .clone();
    let chapter_version = ctx
        .db
        .chapter_versions
        .get(&german_connection)
        .await
        .expect("chapter version should exist");
    let page_ids = chapter_version.pages.clone();
    let pages = ctx
        .db
        .pages
        .get(chapter_version.pages)
        .await
        .expect("page rows should exist");
    assert!(!pages.is_empty());
    for page in &pages {
        let key = format!(
            "mangas/{}/{}/{}/{}.{}",
            manga_id,
            second_version.chapter_id,
            second_version.version_id,
            page.data.page,
            page.data.ext
        );
        ctx.storage
            .reader
            .get(&key, &Default::default())
            .await
            .expect("chapter page should exist in storage");
    }

    ctx.chapter_version
        .delete(&second_version.chapter_id, &second_version.version_id)
        .await
        .expect("version delete should succeed");

    let after_delete = ctx
        .chapter
        .info(&second_version.chapter_id)
        .await
        .expect("chapter info should load");
    assert!(!after_delete
        .versions
        .keys()
        .any(|key| key.ends_with(&second_version.version_id)));
    assert!(matches!(
        ctx.reader.pages(&german_connection).await,
        Err(ApiError::NotFoundInDB)
    ));
    let remaining_pages = ctx
        .db
        .pages
        .get(page_ids)
        .await
        .expect("page lookup should succeed");
    assert!(remaining_pages.is_empty());
    for page in pages {
        let key = format!(
            "mangas/{}/{}/{}/{}.{}",
            manga_id,
            second_version.chapter_id,
            second_version.version_id,
            page.data.page,
            page.data.ext
        );
        let err = match ctx.storage.reader.get(&key, &Default::default()).await {
            Ok(_) => panic!("chapter page should be deleted from storage"),
            Err(err) => err,
        };
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }
}

#[actix_web::test]
async fn adding_new_chapter_recomputes_continue_reading_state() {
    let ctx = TestCtx::new().await;
    let user = ctx
        .register_user("continue", "continue@example.com", "password")
        .await;
    let manga_id = ctx
        .create_manga(&user.id, "Continue Reading Manga", "manga")
        .await;

    let first = ctx.create_chapter(&manga_id, 1.0, "en", 1).await;
    let second = ctx.create_chapter(&manga_id, 2.0, "en", 1).await;

    ctx.reader
        .save_progress(1.0, &first.chapter_id, &user.claim)
        .await
        .expect("completing first chapter should load second");
    ctx.reader
        .save_progress(1.0, &second.chapter_id, &user.claim)
        .await
        .expect("completing latest chapter should succeed");

    let before = ctx
        .reader
        .info(&manga_id, None, &user.claim)
        .await
        .expect("reader info should load");
    assert_eq!(before.open_chapter, second.chapter_id);
    assert!(before.progress >= 0.95);

    let third = ctx.create_chapter(&manga_id, 3.0, "en", 1).await;

    let after = ctx
        .reader
        .info(&manga_id, None, &user.claim)
        .await
        .expect("reader info should load");
    assert_eq!(after.open_chapter, third.chapter_id);
    assert_eq!(after.progress, 0.0);
}

#[actix_web::test]
async fn list_tag_and_kind_actions_interact_with_manga_and_reader_state() {
    let ctx = TestCtx::new().await;
    let user = ctx
        .register_user("lists", "lists@example.com", "password")
        .await;
    let other_user = ctx
        .register_user("lists-2", "lists2@example.com", "password")
        .await;
    let manga_id = ctx.create_manga(&user.id, "List Manga", "webtoon").await;
    let chapter = ctx.create_chapter(&manga_id, 1.0, "en", 1).await;

    ctx.list
        .add("favorites", &user.claim)
        .await
        .expect("list add should succeed");
    ctx.list
        .add_to_list("favorites", &manga_id, &user.claim)
        .await
        .expect("add to list should succeed");

    let lists = ctx
        .list
        .list(&user.claim)
        .await
        .expect("list list should succeed");
    assert!(lists.contains(&"favorites".to_owned()));

    let home = ctx.manga.home(&user.id).await.expect("home should succeed");
    assert!(home
        .favorites
        .iter()
        .any(|entry| entry.manga_id == manga_id));

    ctx.reader
        .save_progress(0.5, &chapter.chapter_id, &user.claim)
        .await
        .expect("progress write should succeed");
    let home_with_progress = ctx.manga.home(&user.id).await.expect("home should succeed");
    assert!(home_with_progress
        .reading
        .iter()
        .any(|entry| entry.manga_id == manga_id));

    let home_other_user = ctx
        .manga
        .home(&other_user.id)
        .await
        .expect("home should succeed");
    assert!(!home_other_user
        .reading
        .iter()
        .any(|entry| entry.manga_id == manga_id));

    ctx.list
        .remove_from_list("favorites", &manga_id, &user.claim)
        .await
        .expect("remove manga from list should succeed");
    ctx.list
        .remove("favorites", &user.claim)
        .await
        .expect("remove list should succeed");

    assert!(matches!(
        ctx.list
            .add_to_list("favorites", "missing-manga-id", &user.claim)
            .await,
        Err(ApiError::NotFoundInDB)
    ));

    let tags = ctx
        .tag
        .search("action")
        .await
        .expect("tag search should succeed");
    assert!(tags.iter().any(|entry| entry.tag == "action"));

    let kinds = ctx.kind.list().await.expect("kind list should succeed");
    assert!(kinds.contains(&"webtoon".to_owned()));
}

#[actix_web::test]
async fn crypto_service_hash_verify_and_claim_roundtrip() {
    let crypto = CryptoService::new(b"secret-for-tests".to_vec());

    let hashed = crypto
        .hash_password("my-password")
        .await
        .expect("hashing should succeed");
    assert!(
        crypto
            .verify_hash("my-password".to_owned(), hashed.clone())
            .await
    );
    assert!(
        !crypto
            .verify_hash("wrong-password".to_owned(), hashed)
            .await
    );

    let token = crypto
        .encode_claim(&Claim::new_access("uid-1".to_owned(), Role::User))
        .expect("encoding should succeed");
    let claim = crypto.get_claim(&token).expect("decoding should succeed");
    assert_eq!(claim.id, "uid-1");
    assert_eq!(claim.role, Role::User);

    let mut expired = Claim::new_access("uid-expired".to_owned(), Role::User);
    expired.exp = (now().as_millis() as u64).saturating_sub(1);
    let expired_token = crypto
        .encode_claim(&expired)
        .expect("expired token should still encode");
    assert!(matches!(
        crypto.get_claim(&expired_token),
        Err(ApiError::ExpiredToken)
    ));
}

#[actix_web::test]
async fn crypto_service_cached_expired_token_returns_without_deadlock() {
    let crypto = Arc::new(CryptoService::new(b"secret-for-tests".to_vec()));
    let mut expired = Claim::new_access("uid-expired-cache".to_owned(), Role::User);
    expired.exp = (now().as_millis() as u64).saturating_sub(1);
    let token = crypto
        .encode_claim(&expired)
        .expect("expired token should encode");
    {
        let mut cache = crypto
            .claims
            .lock()
            .expect("claim cache lock should not be poisoned");
        cache.insert(token.clone(), expired);
    }

    let crypto_clone = crypto.clone();
    let token_clone = token.clone();
    let join = tokio::time::timeout(
        Duration::from_secs(1),
        tokio::task::spawn_blocking(move || crypto_clone.get_claim(&token_clone)),
    )
    .await
    .expect("cached expired token lookup should not deadlock")
    .expect("claim lookup task should finish");

    assert!(matches!(join, Err(ApiError::ExpiredToken)));
}

#[actix_web::test]
async fn token_list_rejects_page_zero() {
    let ctx = TestCtx::new().await;
    assert!(matches!(
        ctx.token.list_tokens(0, 10).await,
        Err(ApiError::InvalidInput(_))
    ));
}

#[actix_web::test]
async fn user_list_rejects_page_zero() {
    let ctx = TestCtx::new().await;
    assert!(matches!(
        ctx.user
            .list(PaginationRequest { page: 0, limit: 10 })
            .await,
        Err(ApiError::InvalidInput(_))
    ));
}

#[actix_web::test]
async fn chapter_version_list_rejects_page_zero() {
    let ctx = TestCtx::new().await;
    assert!(matches!(
        ctx.chapter_version
            .list(PaginationRequest { page: 0, limit: 10 })
            .await,
        Err(ApiError::InvalidInput(_))
    ));
}

#[actix_web::test]
async fn manga_search_rejects_invalid_order() {
    let ctx = TestCtx::new().await;
    let user = ctx
        .register_user("panic", "panic@example.com", "password")
        .await;
    let mut invalid = search_all();
    invalid.order = "no-such-order".to_owned();
    assert!(matches!(
        ctx.manga.search(invalid, &user.id).await,
        Err(ApiError::InvalidInput(_))
    ));
}

#[actix_web::test]
async fn chapter_edit_rejects_invalid_release_date_timestamp() {
    let ctx = TestCtx::new().await;
    let user = ctx
        .register_user("invalid-ts", "invalid-ts@example.com", "password")
        .await;
    let manga_id = ctx
        .create_manga(&user.id, "Invalid Timestamp Manga", "manga")
        .await;
    let chapter = ctx.create_chapter(&manga_id, 1.0, "en", 1).await;

    assert!(matches!(
        ctx.chapter
            .edit(EditChapterRequest {
                chapter_id: chapter.chapter_id,
                titles: None,
                chapter: None,
                tags: None,
                sources: None,
                release_date: Some(i64::MAX as u64),
                clear_release_date: false,
            })
            .await,
        Err(ApiError::InvalidInput(_))
    ));
}

#[actix_web::test]
async fn list_actions_reject_invalid_list_names() {
    let ctx = TestCtx::new().await;
    let user = ctx.register_user("ln", "ln@example.com", "password").await;

    assert!(matches!(
        ctx.list.add("", &user.claim).await,
        Err(ApiError::InvalidInput(_))
    ));
    assert!(matches!(
        ctx.list.add("favorites';--", &user.claim).await,
        Err(ApiError::InvalidInput(_))
    ));
    assert!(matches!(
        ctx.list.remove("..\n", &user.claim).await,
        Err(ApiError::InvalidInput(_))
    ));
}

#[actix_web::test]
async fn reader_rejects_non_finite_progress_values() {
    let ctx = TestCtx::new().await;
    let user = ctx.register_user("np", "np@example.com", "password").await;
    let manga_id = ctx.create_manga(&user.id, "NonFinite", "manga").await;
    let chapter = ctx.create_chapter(&manga_id, 1.0, "en", 1).await;

    assert!(matches!(
        ctx.reader
            .save_progress(f64::NAN, &chapter.chapter_id, &user.claim)
            .await,
        Err(ApiError::InvalidInput(_))
    ));
    assert!(matches!(
        ctx.reader
            .save_progress(f64::INFINITY, &chapter.chapter_id, &user.claim)
            .await,
        Err(ApiError::InvalidInput(_))
    ));
}

#[actix_web::test]
async fn chapter_version_edit_rejects_empty_values() {
    let ctx = TestCtx::new().await;
    let user = ctx
        .register_user("cve", "cve@example.com", "password")
        .await;
    let manga_id = ctx.create_manga(&user.id, "CVE", "manga").await;
    let chapter = ctx.create_chapter(&manga_id, 1.0, "en", 1).await;

    assert!(matches!(
        ctx.chapter_version
            .edit(&chapter.version_id, Some("".to_owned()), None)
            .await,
        Err(ApiError::InvalidInput(_))
    ));
    assert!(matches!(
        ctx.chapter_version
            .edit(&chapter.version_id, None, Some("   ".to_owned()),)
            .await,
        Err(ApiError::InvalidInput(_))
    ));
}

#[actix_web::test]
async fn manga_volume_range_rejects_invalid_bounds() {
    let ctx = TestCtx::new().await;
    let user = ctx
        .register_user("vol", "vol@example.com", "password")
        .await;
    let manga_id = ctx.create_manga(&user.id, "Volumes", "manga").await;

    assert!(matches!(
        ctx.manga
            .set_volume_range(
                &manga_id,
                vec![VolumeRange {
                    start: 10.0,
                    end: Some(5.0),
                    title: Some("bad".to_owned()),
                }],
            )
            .await,
        Err(ApiError::InvalidInput(_))
    ));
    assert!(matches!(
        ctx.manga
            .set_volume_range(
                &manga_id,
                vec![VolumeRange {
                    start: f64::NAN,
                    end: Some(11.0),
                    title: Some("bad".to_owned()),
                }],
            )
            .await,
        Err(ApiError::InvalidInput(_))
    ));
}

#[actix_web::test]
async fn auth_refresh_rejects_access_token() {
    let ctx = TestCtx::new().await;
    let _user = ctx
        .register_user("refresh-guard", "refresh-guard@example.com", "password")
        .await;

    let issued = ctx
        .auth
        .login(LoginRequest::Username(LoginWithUsernameAndPassword {
            username: "refresh-guard".to_owned(),
            password: "password".to_owned(),
        }))
        .await
        .expect("login should succeed");

    assert!(matches!(
        ctx.auth.refresh(&issued.access_token).await,
        Err(ApiError::InvalidInput(_))
    ));
}

#[actix_web::test]
async fn chapter_add_with_invalid_image_handle_keeps_database_clean() {
    let ctx = TestCtx::new().await;
    let user = ctx
        .register_user("chapter-clean", "chapter-clean@example.com", "password")
        .await;
    let manga_id = ctx
        .create_manga(&user.id, "Chapter Clean Manga", "manga")
        .await;

    assert!(matches!(
        ctx.chapter
            .add(
                &manga_id,
                vec!["chapter bad".to_owned()],
                5.0,
                "en",
                vec!["missing-temp-handle".to_owned()],
                vec![],
                vec!["https://source.example".to_owned()],
                None,
            )
            .await,
        Err(ApiError::NotFoundInDB)
    ));

    let manga = ctx
        .db
        .mangas
        .get(&manga_id)
        .await
        .expect("manga should still be readable");
    assert!(manga.chapters.is_empty());
}

#[actix_web::test]
async fn reader_info_rejects_chapter_from_another_manga() {
    let ctx = TestCtx::new().await;
    let user = ctx
        .register_user("reader-boundary", "reader-boundary@example.com", "password")
        .await;
    let manga_a = ctx.create_manga(&user.id, "Reader A", "manga").await;
    let manga_b = ctx.create_manga(&user.id, "Reader B", "manga").await;

    let _chapter_a = ctx.create_chapter(&manga_a, 1.0, "en", 1).await;
    let chapter_b = ctx.create_chapter(&manga_b, 1.0, "en", 1).await;

    assert!(matches!(
        ctx.reader
            .info(&manga_a, Some(chapter_b.chapter_id), &user.claim)
            .await,
        Err(ApiError::NotFoundInDB)
    ));
}

#[actix_web::test]
async fn manga_remove_cover_rejects_removing_last_cover() {
    let ctx = TestCtx::new().await;
    let user = ctx
        .register_user("cover-guard", "cover-guard@example.com", "password")
        .await;
    let manga_id = ctx.create_manga(&user.id, "Cover Guard", "manga").await;

    assert!(matches!(
        ctx.manga.remove_cover(&manga_id, 0).await,
        Err(ApiError::InvalidInput(_))
    ));
}

#[actix_web::test]
async fn manga_add_relation_requires_existing_relation_target() {
    let ctx = TestCtx::new().await;
    let user = ctx
        .register_user("relation-guard", "relation-guard@example.com", "password")
        .await;
    let manga_id = ctx.create_manga(&user.id, "Relation Guard", "manga").await;

    assert!(matches!(
        ctx.manga.add_relation(&manga_id, "missing-manga").await,
        Err(ApiError::NotFoundInDB)
    ));

    let info = ctx
        .manga
        .info(manga_id, &user.id)
        .await
        .expect("manga info should still load");
    assert!(info.relations.is_empty());
}

#[actix_web::test]
async fn manga_create_rejects_blank_nested_values() {
    let ctx = TestCtx::new().await;
    let user = ctx
        .register_user("nested", "nested@example.com", "password")
        .await;
    let cover = ctx.upload_png().await;

    assert!(matches!(
        ctx.manga
            .create(
                AddMangaRequest {
                    names: HashMap::from([(
                        "en".to_owned(),
                        StringList {
                            items: vec!["   ".to_owned()]
                        }
                    )]),
                    kind: "manga".to_owned(),
                    status: Status::Ongoing,
                    description: None,
                    tags: vec![],
                    image_temp_name: cover,
                    authors: vec!["author".to_owned()],
                    publishers: vec!["publisher".to_owned()],
                    artists: vec!["artist".to_owned()],
                    sources: vec!["https://source.example".to_owned()],
                    scrapers: vec![v1::Scraper {
                        channel: "en".to_owned(),
                        url: "https://scraper.example/feed".to_owned(),
                    }],
                },
                &user.id,
            )
            .await,
        Err(ApiError::InvalidInput(_))
    ));
}

#[actix_web::test]
async fn chapter_add_rejects_empty_titles_and_sources() {
    let ctx = TestCtx::new().await;
    let user = ctx
        .register_user(
            "chapter-validate",
            "chapter-validate@example.com",
            "password",
        )
        .await;
    let manga_id = ctx
        .create_manga(&user.id, "Chapter Validate Manga", "manga")
        .await;

    assert!(matches!(
        ctx.chapter
            .add(
                &manga_id,
                vec![],
                1.0,
                "en",
                vec![ctx.upload_png().await],
                vec![],
                vec!["https://source.example".to_owned()],
                None,
            )
            .await,
        Err(ApiError::InvalidInput(_))
    ));

    assert!(matches!(
        ctx.chapter
            .add(
                &manga_id,
                vec!["chapter 1".to_owned()],
                1.0,
                "en",
                vec![ctx.upload_png().await],
                vec![],
                vec![" ".to_owned()],
                None,
            )
            .await,
        Err(ApiError::InvalidInput(_))
    ));
}
