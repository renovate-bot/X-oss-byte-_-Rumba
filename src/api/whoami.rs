use actix_identity::Identity;

use serde::Serialize;

use crate::api::error::ApiError;
use crate::api::settings::SettingUpdateRequest;
use crate::db;
use crate::db::Pool;
use actix_web::{web, HttpRequest, HttpResponse};

#[derive(Serialize)]
pub struct GeoInfo {
    country: String,
}

#[derive(Serialize, Default)]
pub struct WhoamiResponse {
    geo: Option<GeoInfo>,
    // #[deprecated(note="Confusing name. We should consider just changing to user_id")]
    username: Option<String>,
    is_authenticated: Option<bool>,
    email: Option<String>,
    avatar_url: Option<String>,
    is_subscriber: Option<bool>,
    subscription_type: Option<String>,
    settings: Option<SettingUpdateRequest>,
}

const CLOUDFRONT_COUNTRY_HEADER: &str = "CloudFront-Viewer-Country-Name";

pub async fn whoami(
    _req: HttpRequest,
    id: Identity,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ApiError> {
    let header_info = _req.headers().get(CLOUDFRONT_COUNTRY_HEADER);

    let country = header_info.map(|header| GeoInfo {
        country: String::from(header.to_str().unwrap_or("Unknown")),
    });

    match id.identity() {
        Some(id) => {
            let mut conn_pool = pool.get()?;
            let user = db::users::get_user(&mut conn_pool, id).await;
            match user {
                Ok(found) => {
                    let settings = db::settings::get_settings(&mut conn_pool, &found)?;
                    let response = WhoamiResponse {
                        geo: country,
                        username: Option::Some(found.fxa_uid),
                        subscription_type: Option::Some(
                            found.subscription_type.unwrap_or_default().into(),
                        ),
                        avatar_url: found.avatar_url,
                        is_subscriber: Option::Some(found.is_subscriber),
                        is_authenticated: Option::Some(true),
                        email: Option::Some(found.email),
                        settings: settings.map(Into::into),
                    };
                    Ok(HttpResponse::Ok().json(response))
                }
                Err(_err) => Err(ApiError::InvalidSession),
            }
        }
        None => {
            let res = WhoamiResponse {
                geo: country,
                ..Default::default()
            };
            Ok(HttpResponse::Ok().json(res))
        }
    }
}
