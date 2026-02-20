use std::sync::Arc;

use api_structure::v1::{
    Character as ApiCharacter, CreateCharacterRequest, OptionalString, SearchRequest,
};
use db::{
    auth::RecordData,
    character::{Character, CharacterDBService},
};

use crate::error::{ApiError, ApiResult};

pub struct CharacterActions {
    pub characters: Arc<CharacterDBService>,
}

fn to_api_character(value: RecordData<Character>) -> ApiCharacter {
    ApiCharacter {
        id: value.id.id().to_string(),
        imgs: value
            .data
            .imgs
            .into_iter()
            .map(OptionalString::from)
            .collect(),
        names: value.data.names,
        description: value.data.description,
        sex: value.data.sex,
        links: value.data.links,
    }
}

impl CharacterActions {
    fn validate_non_empty_items(field: &str, items: &[String]) -> ApiResult<()> {
        if items.iter().any(|value| value.trim().is_empty()) {
            return Err(ApiError::invalid_input(&format!(
                "{field} cannot contain empty values"
            )));
        }
        Ok(())
    }

    pub async fn search(&self, data: SearchRequest) -> ApiResult<Vec<ApiCharacter>> {
        if data.page == 0 {
            return Err(ApiError::invalid_input("page must be >= 1"));
        }
        if data.limit == 0 {
            return Err(ApiError::invalid_input("limit must be >= 1"));
        }
        if data.query.trim().is_empty() {
            return Err(ApiError::invalid_input("query cannot be empty"));
        }
        let values = self
            .characters
            .search(&data.query, data.page, data.limit)
            .await?;
        Ok(values.into_iter().map(to_api_character).collect())
    }

    pub async fn info(&self, id: &str) -> ApiResult<ApiCharacter> {
        if id.trim().is_empty() {
            return Err(ApiError::invalid_input("id cannot be empty"));
        }
        let value = self.characters.info(id).await?;
        Ok(to_api_character(value))
    }

    pub async fn create(&self, data: CreateCharacterRequest) -> ApiResult<String> {
        if data.names.is_empty() {
            return Err(ApiError::invalid_input("names cannot be empty"));
        }
        Self::validate_non_empty_items("names", &data.names)?;
        Self::validate_non_empty_items("links", &data.links)?;
        if data.imgs.iter().any(|img| {
            img.value
                .as_ref()
                .map(|v| v.trim().is_empty())
                .unwrap_or(false)
        }) {
            return Err(ApiError::invalid_input("imgs cannot contain empty strings"));
        }

        let character = Character {
            imgs: data.imgs.into_iter().map(Into::into).collect(),
            names: data.names,
            description: data.description,
            sex: data.sex,
            links: data.links,
            updated: Default::default(),
            created: Default::default(),
        };
        let id = self.characters.create(character).await?;
        Ok(id.id().to_string())
    }
}
