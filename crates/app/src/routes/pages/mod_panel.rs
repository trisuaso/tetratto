use super::{PaginatedQuery, render_error};
use crate::{State, assets::initial_context, get_lang, get_user_from_token};
use axum::{
    extract::{Path, Query},
    response::{Html, IntoResponse},
    Extension,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use tetratto_core::model::{Error, permissions::FinePermission, reactions::AssetType};

/// `/mod_panel/audit_log`
pub async fn audit_log_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Query(req): Query<PaginatedQuery>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = match get_user_from_token!(jar, data.0) {
        Some(ua) => ua,
        None => {
            return Err(Html(
                render_error(Error::NotAllowed, &jar, &data, &None).await,
            ));
        }
    };

    if !user.permissions.check(FinePermission::VIEW_AUDIT_LOG) {
        return Err(Html(
            render_error(Error::NotAllowed, &jar, &data, &None).await,
        ));
    }

    let items = match data.0.get_audit_log_entries(12, req.page).await {
        Ok(p) => p,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &Some(user)).await)),
    };

    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &Some(user)).await;
    context.insert("items", &items);
    context.insert("page", &req.page);

    // return
    Ok(Html(data.1.render("mod/audit_log.html", &context).unwrap()))
}

/// `/mod_panel/reports`
pub async fn reports_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Query(req): Query<PaginatedQuery>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = match get_user_from_token!(jar, data.0) {
        Some(ua) => ua,
        None => {
            return Err(Html(
                render_error(Error::NotAllowed, &jar, &data, &None).await,
            ));
        }
    };

    if !user.permissions.check(FinePermission::VIEW_REPORTS) {
        return Err(Html(
            render_error(Error::NotAllowed, &jar, &data, &None).await,
        ));
    }

    let items = match data.0.get_reports(12, req.page).await {
        Ok(p) => p,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &Some(user)).await)),
    };

    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &Some(user)).await;
    context.insert("items", &items);
    context.insert("page", &req.page);

    // return
    Ok(Html(data.1.render("mod/reports.html", &context).unwrap()))
}

#[derive(Deserialize)]
pub struct FileReportQuery {
    pub asset: String,
    pub asset_type: AssetType,
}

/// `/mod_panel/file_report`
pub async fn file_report_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Query(req): Query<FileReportQuery>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = match get_user_from_token!(jar, data.0) {
        Some(ua) => ua,
        None => {
            return Err(Html(
                render_error(Error::NotAllowed, &jar, &data, &None).await,
            ));
        }
    };

    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &Some(user)).await;
    context.insert("asset", &req.asset);
    context.insert("asset_type", &req.asset_type);

    // return
    Ok(Html(
        data.1.render("mod/file_report.html", &context).unwrap(),
    ))
}

/// `/mod_panel/ip_bans`
pub async fn ip_bans_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Query(req): Query<PaginatedQuery>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = match get_user_from_token!(jar, data.0) {
        Some(ua) => ua,
        None => {
            return Err(Html(
                render_error(Error::NotAllowed, &jar, &data, &None).await,
            ));
        }
    };

    if !user.permissions.check(FinePermission::MANAGE_BANS) {
        return Err(Html(
            render_error(Error::NotAllowed, &jar, &data, &None).await,
        ));
    }

    let items = match data.0.get_ipbans(12, req.page).await {
        Ok(p) => p,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &Some(user)).await)),
    };

    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &Some(user)).await;
    context.insert("items", &items);
    context.insert("page", &req.page);

    // return
    Ok(Html(data.1.render("mod/ip_bans.html", &context).unwrap()))
}

/// `/mod_panel/profile/{id}`
pub async fn manage_profile_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(id): Path<usize>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = match get_user_from_token!(jar, data.0) {
        Some(ua) => ua,
        None => {
            return Err(Html(
                render_error(Error::NotAllowed, &jar, &data, &None).await,
            ));
        }
    };

    if !user.permissions.check(FinePermission::MANAGE_USERS) {
        return Err(Html(
            render_error(Error::NotAllowed, &jar, &data, &None).await,
        ));
    }

    let profile = match data.0.get_user_by_id(id).await {
        Ok(p) => p,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &Some(user)).await)),
    };

    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &Some(user)).await;
    context.insert("profile", &profile);

    // return
    Ok(Html(data.1.render("mod/profile.html", &context).unwrap()))
}

/// `/mod_panel/profile/{id}/warnings`
pub async fn manage_profile_warnings_request(
    jar: CookieJar,
    Extension(data): Extension<State>,
    Path(id): Path<usize>,
    Query(req): Query<PaginatedQuery>,
) -> impl IntoResponse {
    let data = data.read().await;
    let user = match get_user_from_token!(jar, data.0) {
        Some(ua) => ua,
        None => {
            return Err(Html(
                render_error(Error::NotAllowed, &jar, &data, &None).await,
            ));
        }
    };

    if !user.permissions.check(FinePermission::MANAGE_USERS) {
        return Err(Html(
            render_error(Error::NotAllowed, &jar, &data, &None).await,
        ));
    }

    let profile = match data.0.get_user_by_id(id).await {
        Ok(p) => p,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &Some(user)).await)),
    };

    let list = match data
        .0
        .get_user_warnings_by_user(profile.id, 12, req.page)
        .await
    {
        Ok(p) => p,
        Err(e) => return Err(Html(render_error(e, &jar, &data, &Some(user)).await)),
    };

    let lang = get_lang!(jar, data.0);
    let mut context = initial_context(&data.0.0, lang, &Some(user)).await;

    context.insert("profile", &profile);
    context.insert("items", &list);
    context.insert("page", &req.page);

    // return
    Ok(Html(data.1.render("mod/warnings.html", &context).unwrap()))
}
