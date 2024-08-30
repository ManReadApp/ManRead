use super::role::Role;

/// NotVerified will be used to reset the password
pub struct Kind {
    pub single: bool,
    pub kind: Role,
}
impl Kind {
    pub fn new(single: bool, kind: Role) -> Self {
        Self { single, kind }
    }
}

impl TryFrom<u32> for Kind {
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
            (Role::from(d), chars.next().ok_or(())?)
        };

        let single = matches!(single, '1');
        Ok(Self { single, kind: role })
    }
}

impl From<Kind> for u32 {
    fn from(value: Kind) -> Self {
        let f = (value.kind as u8).to_string();
        let s = (value.single as u8).to_string();
        format!("{f}{s}").parse().expect("cant fail")
    }
}
