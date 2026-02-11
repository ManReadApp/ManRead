use std::time::{Duration, SystemTime};

use actix_web::{
    error::ErrorInternalServerError,
    http::header::{
        self, ETag, EntityTag, Header, HttpDate, IfModifiedSince, IfNoneMatch, LastModified,
    },
    web, HttpRequest, HttpResponse,
};
use chrono::{DateTime, Utc};
use storage::{Object, StorageSystem};

fn normalize_etag(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.starts_with("W/\"") || trimmed.starts_with('"') {
        trimmed.to_string()
    } else {
        format!("\"{}\"", trimmed)
    }
}

#[inline]
fn weak_etag_eq(a: &EntityTag, b: &EntityTag) -> bool {
    a.tag() == b.tag()
}

pub fn maybe_not_modified(
    req: &HttpRequest,
    obj_etag: Option<&str>,
    obj_last_modified: Option<SystemTime>,
) -> Option<HttpResponse> {
    let entity_tag: Option<EntityTag> = obj_etag
        .map(normalize_etag)
        .and_then(|s| s.parse::<EntityTag>().ok());

    let last_modified: Option<HttpDate> = obj_last_modified.map(|dt| HttpDate::from(dt));
    let inm = IfNoneMatch::parse(req).ok();
    let ims = IfModifiedSince::parse(req).ok();

    if let (Some(etag), Some(inm)) = (entity_tag.as_ref(), inm) {
        if match inm {
            IfNoneMatch::Any => true,
            IfNoneMatch::Items(tags) => tags.iter().any(|t| weak_etag_eq(t, etag)),
        } {
            let mut resp = HttpResponse::NotModified();
            resp.insert_header(ETag(etag.clone()));
            if let Some(lm) = last_modified {
                resp.insert_header(LastModified(lm));
            }
            return Some(resp.finish());
        }

        return None;
    }

    if let (Some(lm), Some(ims)) = (last_modified, ims) {
        if lm <= ims.0 {
            let mut resp = HttpResponse::NotModified();
            resp.insert_header(LastModified(lm));
            return Some(resp.finish());
        }
    }

    None
}

pub async fn download(
    storage: web::Data<StorageSystem>,
    path: web::Path<String>,
    req: &HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let key = path.into_inner();
    let obj = storage
        .reader
        .get(&key, &Default::default())
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(stream(req, obj, true))
}

fn stream(req: &HttpRequest, obj: Object, cache: bool) -> HttpResponse {
    if let Some(resp) = maybe_not_modified(req, obj.etag.as_deref(), obj.last_modified) {
        return resp;
    }
    let mut resp = HttpResponse::Ok();
    if let Some(ct) = obj.content_type {
        resp.content_type(ct.to_string());
    }
    if let Some(len) = obj.content_length {
        resp.insert_header(("Content-Length", len.to_string()));
    }
    if cache {
        if let Some(etag) = obj.etag {
            resp.insert_header(("ETag", etag));
        }

        if let Some(lm) = obj.last_modified {
            resp.insert_header(("Last-Modified", LastModified(lm.into())));
        }
    } else {
        resp.insert_header((
            header::CACHE_CONTROL,
            "no-store, no-cache, must-revalidate, max-age=0",
        ))
        .insert_header((header::PRAGMA, "no-cache"))
        .insert_header((header::EXPIRES, "0"));
    }

    resp.streaming(obj.stream)
}
