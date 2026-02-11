pub mod error;
pub mod models;
pub mod req;

pub mod resp;
pub mod search;

use crate::v1::{
    ActivationTokenKind, Claim, Gender, JwtType, MangaReaderResponse, OptionalString,
    ReaderChapter, ReaderPage, Role, Status, StringList, Tag, TagSex,
};

use std::collections::HashSet;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/v1.rs"));
}

impl From<Vec<String>> for StringList {
    fn from(value: Vec<String>) -> Self {
        Self { items: value }
    }
}

impl From<StringList> for Vec<String> {
    fn from(value: StringList) -> Self {
        value.items
    }
}

impl From<Option<String>> for OptionalString {
    fn from(value: Option<String>) -> Self {
        Self { value }
    }
}

impl From<OptionalString> for Option<String> {
    fn from(value: OptionalString) -> Self {
        value.value
    }
}

impl MangaReaderResponse {
    pub fn get_chapter(&self, id: &str) -> Option<&ReaderChapter> {
        self.chapters.iter().find(|&ch| ch.chapter_id == id)
    }

    pub fn get_prev_chapter(&self, id: &str) -> Option<&ReaderChapter> {
        let mut last = None;
        for ch in &self.chapters {
            if ch.chapter_id == id {
                break;
            }
            last = Some(&ch.chapter_id)
        }
        match last {
            None => None,
            Some(v) => self.get_chapter(v),
        }
    }

    pub fn get_next_chapter(&self, id: &str) -> Option<&ReaderChapter> {
        let mut hit = false;
        for ch in &self.chapters {
            if hit {
                return Some(ch);
            }
            if ch.chapter_id == id {
                hit = true;
            }
        }
        None
    }
}

impl MangaReaderResponse {
    pub fn no_chapters(&self) -> bool {
        self.chapters.is_empty()
    }
    pub fn missing_chapters(&self) -> Vec<f64> {
        let ch = self.chapters.iter().map(|v| v.chapter).collect::<Vec<_>>();
        let max = max_f64(&ch);

        let ch = ch
            .into_iter()
            .map(|v| v.to_string())
            .collect::<HashSet<_>>();
        let mut missing = vec![];
        if let Some(v) = max {
            for num in 1..v.floor() as u32 {
                let num = (num as f64).to_string();
                if !ch.contains(&num) {
                    missing.push(num);
                }
            }
        }
        missing
            .into_iter()
            .map(|v| v.parse().expect("cant fail. f64 => to_string => f64"))
            .collect()
    }
}

fn max_f64(items: &Vec<f64>) -> Option<f64> {
    let mut max = None;
    for item in items {
        if let Some(max) = &mut max {
            if item > max {
                *max = *item;
            }
        } else {
            max = Some(*item)
        }
    }
    max
}

impl ReaderPage {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            page_id: "".to_string(),
            width: w,
            height: h,
            ext: "gif".to_string(),
            translation: false,
            progress: None,
        }
    }
    pub fn width(&self, available_height: f32) -> f32 {
        (available_height / self.height as f32) * self.width as f32
    }
    pub fn height(&self, available_width: f32) -> f32 {
        (available_width / self.width as f32) * self.height as f32
    }
}

impl ActivationTokenKind {
    pub fn new(single: bool, kind: Role) -> Self {
        Self { single, kind }
    }
}

impl TryFrom<u32> for ActivationTokenKind {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let s = value.to_string();
        let (role, single) = if s.len() == 1 {
            (Role::NotVerified, s.chars().next().ok_or(())?)
        } else {
            assert_eq!(s.len(), 2);
            let mut chars = s.chars();
            let d: u32 = chars
                .next()
                .ok_or(())?
                .to_string()
                .parse()
                .map_err(|_| ())?;
            (Role::try_from(d).unwrap(), chars.next().ok_or(())?)
        };

        let single = matches!(single, '1');
        Ok(Self { single, kind: role })
    }
}

impl From<ActivationTokenKind> for u32 {
    fn from(value: ActivationTokenKind) -> Self {
        let f = (value.kind as u32).to_string();
        let s = (value.single as u8).to_string();
        format!("{f}{s}").parse().expect("cant fail")
    }
}

impl TryFrom<u64> for Gender {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        v1::Gender::from_i32(value as i32).ok_or(())
    }
}

impl TryFrom<u64> for v1::Visibility {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        v1::Visibility::from_i32(value as i32).ok_or(())
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.sex, self.tag)
    }
}

impl Display for TagSex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TagSex::Female => write!(f, "♀"),
            TagSex::Male => write!(f, "♂"),
            TagSex::Both => write!(f, "⚤"), //⚥
            TagSex::None => write!(f, ""),
            TagSex::FemaleMale => write!(f, "♀→♂"),
            TagSex::MaleFemale => write!(f, "♂→♀"),
            TagSex::Unknown => write!(f, "⚧"),
        }
    }
}

impl TagSex {
    pub fn get_name(&self) -> &'static str {
        match self {
            TagSex::Female => "female",
            TagSex::Male => "male",
            TagSex::Both => "both",
            TagSex::None => "none",
            TagSex::FemaleMale => "female to male",
            TagSex::MaleFemale => "male to female",
            TagSex::Unknown => "i don't know",
        }
    }
    pub fn get_all() -> Vec<TagSex> {
        [
            TagSex::Female,
            TagSex::Male,
            TagSex::Both,
            TagSex::None,
            TagSex::FemaleMale,
            TagSex::MaleFemale,
            TagSex::Unknown,
        ]
        .to_vec()
    }
}

impl TryFrom<u64> for TagSex {
    type Error = ();
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        v1::TagSex::from_i32(value as i32).ok_or(())
    }
}

impl From<Status> for u64 {
    fn from(value: Status) -> Self {
        i32::from(value) as u64
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Status::Dropped => "Dropped",
                Status::Hiatus => "Hiatus",
                Status::Ongoing => "Ongoing",
                Status::Completed => "Completed",
                Status::Upcoming => "Upcoming",
            }
        )
    }
}

impl TryFrom<u64> for Status {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::from_i32(value as i32).ok_or(())
    }
}

impl Role {
    pub fn get_permissions(&self) -> Vec<Permission> {
        match self {
            Role::NotVerified => vec![Permission::Verify],
            Role::User => vec![Permission::Read],
            Role::Author => vec![
                Permission::Read,
                Permission::Create,
                Permission::RequestDelete,
            ],
            Role::Moderator => vec![
                Permission::Read,
                Permission::Create,
                Permission::RequestDelete,
                Permission::Review,
            ],
            Role::CoAdmin => vec![
                Permission::Read,
                Permission::Create,
                Permission::RequestDelete,
                Permission::Review,
            ],
            Role::Admin => vec![
                Permission::Read,
                Permission::Create,
                Permission::Review,
                Permission::RequestDelete,
                Permission::Impersonate,
            ],
        }
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum Permission {
    None, //used
    Verify,
    Read,   // used
    Create, // used
    Review,
    Delete,
    RequestDelete, // used
    Impersonate,
    ManageExternalServices,
}

impl TryFrom<u32> for Role {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::from_i32(value as i32).ok_or(())
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::NotVerified => write!(f, "Undefined"),
            Role::User => write!(f, "User"),
            Role::Moderator => write!(f, "Moderator"),
            Role::CoAdmin => write!(f, "Co-Admin"),
            Role::Admin => write!(f, "Admin"),
            Role::Author => write!(f, "Author"),
        }
    }
}

impl FromStr for Role {
    type Err = ();

    fn from_str(role: &str) -> Result<Self, Self::Err> {
        Ok(match role {
            "Author" => Self::Author,
            "Admin" => Self::Admin,
            "Co-Admin" => Self::CoAdmin,
            "Moderator" => Self::Moderator,
            "User" => Self::User,
            _ => Self::NotVerified,
        })
    }
}

impl Claim {
    pub fn new(uid: String, role: Role, jwt_type: JwtType, dur: Duration) -> Self {
        let expiration = now() + dur;

        Claim {
            id: uid,
            role,
            exp: expiration.as_millis() as u64,
            r#type: jwt_type,
        }
    }

    pub fn new_access(uid: String, role: Role) -> Self {
        Self::new(uid, role, JwtType::AccessToken, Duration::from_secs(120)) //2min
    }

    pub fn new_refresh(uid: String, role: Role) -> Self {
        Self::new(
            uid,
            role,
            JwtType::RefreshToken,
            Duration::from_secs(REFRESH_SECS),
        ) // 60days
    }
}

pub const REFRESH_SECS: u64 = 60 * 60 * 24 * 60;

pub fn now() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
}
