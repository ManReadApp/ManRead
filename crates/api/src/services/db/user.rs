use crate::errors::ApiResult;
use api_structure::error::{ApiErr, ApiErrorType};
use api_structure::models::auth::gender::Gender;
use api_structure::models::auth::role::Role;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use surrealdb::engine::local::Db;
use surrealdb::opt::PatchOp;
use surrealdb::sql::{Datetime, Thing};
use surrealdb::{Error, Surreal};
use surrealdb_extras::{
    Record, RecordData, SurrealSelect, SurrealTable, SurrealTableInfo, ThingFunc, ThingType,
};

#[derive(SurrealTable, Serialize, Deserialize, Debug)]
#[db("users")]
#[sql(["DEFINE EVENT user_updated ON TABLE users WHEN $event = \"UPDATE\" AND $before.updated == $after.updated THEN (UPDATE $after.id SET updated = time::now() );"])]
pub struct User {
    pub names: Vec<String>,
    pub email: String,
    pub password: String,
    pub role: u32,
    pub bio: Option<String>,
    pub links: Vec<String>,
    pub thumb_ext: Option<String>,
    pub icon_ext: Option<String>,
    pub birthdate: Datetime,
    pub gender: u32,
    #[opt(exclude = true)]
    pub updated: Datetime,
    #[opt(exclude = true)]
    pub created: Datetime,
}

#[derive(SurrealSelect, Deserialize)]
struct Empty {}

#[derive(SurrealSelect, Deserialize, Serialize)]
struct UserRole {
    role: u32,
}

#[derive(SurrealSelect, Deserialize)]
pub struct UserRolePassword {
    pub role: u32,
    pub password: String,
}

pub struct UserDBService {
    pub conn: Arc<Surreal<Db>>,
    temp: Arc<Mutex<HashMap<String, String>>>,
}

impl UserDBService {
    #[cfg(feature = "dev")]
    pub async fn all(&self) -> ApiResult<Vec<String>> {
        let data: Vec<RecordData<User>> = User::all(&self.conn).await?;
        Ok(data.into_iter().map(|v| v.data.names).flatten().collect())
    }

    pub async fn get_username(&self, id: &str) -> Option<String> {
        if let Some(v) = self.temp.lock().unwrap().get(id) {
            return Some(v.clone());
        }
        let mut hm = HashMap::new();
        let res: Vec<RecordData<User>> = User::all(&*self.conn).await.ok()?;
        for mut item in res {
            hm.insert(item.id.id().to_string(), item.data.names.remove(0));
        }
        let v = hm.get(id).cloned();
        *self.temp.lock().unwrap() = hm;
        v
    }

    pub async fn get_id(&self, ident: &str, email: bool) -> ApiResult<String> {
        let search = Self::emailusername_query(email, ident);
        let mut user = User::search(&*self.conn, Some(search)).await?;
        if user.is_empty() {
            return Err(ApiErr {
                message: Some("No user found".to_string()),
                cause: None,
                err_type: ApiErrorType::InvalidInput,
            }
            .into());
        }
        let user: RecordData<Empty> = user.remove(0);
        Ok(user.id.id().to_string())
    }

    pub async fn set_password(&self, id: &str, password: String) -> ApiResult<()> {
        let v: ThingFunc = ThingFunc::new(Thing::from((User::name(), id)));
        let _: Option<Record> = v
            .patch(&*self.conn, PatchOp::replace("password", password))
            .await?;
        Ok(())
    }

    pub async fn set_role(&self, id: &str, role: Role) -> ApiResult<()> {
        let v: ThingFunc = ThingFunc::new(Thing::from((User::name(), id)));
        let role = UserRole { role: role as u32 };
        let _: Option<Record> = v.update(&*self.conn, role).await?;
        Ok(())
    }

    fn emailusername_query(email: bool, search: &str) -> String {
        match email {
            true => format!("WHERE email = \"{}\"", search.to_lowercase()),
            false => format!("WHERE names CONTAINS \"{}\"", search),
        }
    }
    pub async fn login_data(
        &self,
        search: &str,
        email: bool,
    ) -> ApiResult<RecordData<UserRolePassword>> {
        let search = Self::emailusername_query(email, search);
        let mut user = User::search(&*self.conn, Some(search)).await?;
        if user.is_empty() {
            return Err(ApiErr {
                message: Some("Couldnt find user".to_string()),
                cause: None,
                err_type: ApiErrorType::InvalidInput,
            }
            .into());
        }
        Ok(user.remove(0))
    }
    pub async fn get_role(&self, id: &str) -> ApiResult<Role> {
        let v: ThingType<User> = ThingType::new(ThingFunc::new(Thing::from((User::name(), id))));
        let v: RecordData<UserRole> = match v.get_part(&*self.conn).await? {
            Some(v) => v,
            None => {
                return Err(ApiErr {
                    message: Some("Failed to find user in db".to_string()),
                    cause: None,
                    err_type: ApiErrorType::ReadError,
                }
                .into())
            }
        };
        let role = v.data.role;
        Ok(Role::from(role))
    }
    pub fn new(conn: Arc<Surreal<Db>>) -> Self {
        Self {
            conn,
            temp: Default::default(),
        }
    }

    pub async fn new_user(
        &self,
        name: String,
        email: String,
        password: String,
        icon_ext: String,
        bd: NaiveDate,
        gender: Gender,
    ) -> Result<Record, Error> {
        let datetime =
            DateTime::from_naive_utc_and_offset(NaiveDateTime::new(bd, NaiveTime::MIN), Utc);
        let birthdate = Datetime::from(datetime);
        let gender = gender as u32;

        let user = User {
            names: vec![name],
            email,
            password,
            role: Role::NotVerified as u32,
            bio: None,
            links: vec![],
            thumb_ext: None,
            icon_ext: Some(icon_ext),
            birthdate,
            gender,
            updated: Default::default(),
            created: Default::default(),
        };

        user.add_i(&*self.conn).await
    }

    pub async fn email_exists(&self, email: &str) -> bool {
        let result: Vec<RecordData<Empty>> =
            User::search(&*self.conn, Some(format!("WHERE email = \"{}\"", email)))
                .await
                .unwrap_or_default();
        !result.is_empty()
    }

    pub async fn username_exists(&self, name: &str) -> bool {
        let result: Vec<RecordData<Empty>> = User::search(
            &*self.conn,
            Some(format!("WHERE names CONTAINS \"{}\"", name)),
        )
        .await
        .unwrap_or_default();
        !result.is_empty()
    }
}
