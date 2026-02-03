use actix_web::web::{Data, ReqData};
use api_structure::models::auth::jwt::Claim;
use apistos::{actix::CreatedJson, api_operation};

use crate::{error::ApiResult, models::user::UserDBService};

#[api_operation(
    tag = "auth",
    summary = "Logs out every device of the user",
    description = r###"Add a new pet to the store
    Plop"###
)]
pub(crate) async fn exec(
    user_service: Data<UserDBService>,
    claim: ReqData<Claim>,
) -> ApiResult<CreatedJson<u8>> {
    user_service.logout(&claim.id).await?;
    Ok(CreatedJson(0))
}

pub fn register() -> apistos::web::Resource {
    apistos::web::resource("/sign-out").route(apistos::web::delete().to(exec))
}
