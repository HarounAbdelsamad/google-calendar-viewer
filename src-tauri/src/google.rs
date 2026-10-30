//! Read-only Google Calendar API v3 client.

use futures::future::try_join_all;
use reqwest::{RequestBuilder, StatusCode, Url};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::error::{Error, Result};

const API: &str = "https://www.googleapis.com/calendar/v3";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Calendar {
    id: String,
    summary: String,
    background_color: String,
    foreground_color: String,
    primary: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    id: String,
    calendar_id: String,
    title: String,
    start: String,
    end: Option<String>,
    all_day: bool,
    location: Option<String>,
    description: Option<String>,
    html_link: Option<String>,
    meet_link: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Page<T> {
    #[serde(default = "Vec::new")]
    items: Vec<T>,
    next_page_token: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiCalendar {
    id: String,
    summary: Option<String>,
    summary_override: Option<String>,
    background_color: Option<String>,
    foreground_color: Option<String>,
    #[serde(default)]
    primary: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiEvent {
    id: String,
    status: Option<String>,
    summary: Option<String>,
    description: Option<String>,
    location: Option<String>,
    html_link: Option<String>,
    hangout_link: Option<String>,
    start: Option<EventTime>,
    end: Option<EventTime>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EventTime {
    date: Option<String>,
    date_time: Option<String>,
}

async fn send<T: DeserializeOwned>(req: RequestBuilder) -> Result<T> {
    let res = req.send().await?;
    match res.status() {
        s if s.is_success() => Ok(res.json().await?),
        StatusCode::UNAUTHORIZED => Err(Error::AuthExpired),
        s => {
            let body = res.text().await.unwrap_or_default();
            if s == StatusCode::TOO_MANY_REQUESTS || body.contains("rateLimitExceeded") {
                return Err(Error::Other(
                    "Google is rate-limiting requests. Wait a minute, then press ↻.".into(),
                ));
            }
            Err(Error::Other(format!("Google API error {s}: {body}")))
        }
    }
}

/// Fetches every page of a list endpoint.
async fn fetch_all<T: DeserializeOwned>(
    http: &reqwest::Client,
    token: &str,
    url: Url,
    query: &[(&str, &str)],
) -> Result<Vec<T>> {
    let mut items = Vec::new();
    let mut page_token: Option<String> = None;
    loop {
        let mut req = http.get(url.clone()).bearer_auth(token).query(query);
        if let Some(p) = &page_token {
            req = req.query(&[("pageToken", p)]);
        }
        let page: Page<T> = send(req).await?;
        items.extend(page.items);
        match page.next_page_token {
            Some(p) => page_token = Some(p),
            None => return Ok(items),
        }
    }
}

pub async fn list_calendars(http: &reqwest::Client, token: &str) -> Result<Vec<Calendar>> {
    let url = Url::parse(&format!("{API}/users/me/calendarList")).unwrap();
    let items: Vec<ApiCalendar> = fetch_all(http, token, url, &[("maxResults", "250")]).await?;
    Ok(items
        .into_iter()
        .map(|c| Calendar {
            summary: c.summary_override.or(c.summary).unwrap_or_else(|| c.id.clone()),
            id: c.id,
            background_color: c.background_color.unwrap_or_else(|| "#4285f4".into()),
            foreground_color: c.foreground_color.unwrap_or_else(|| "#ffffff".into()),
            primary: c.primary,
        })
        .collect())
}

pub async fn list_events(
    http: &reqwest::Client,
    token: &str,
    calendar_ids: &[String],
    time_min: &str,
    time_max: &str,
) -> Result<Vec<Event>> {
    let per_calendar = calendar_ids.iter().map(|id| async move {
        let mut url = Url::parse(API).unwrap();
        url.path_segments_mut().unwrap().push("calendars").push(id).push("events");
        let items: Vec<ApiEvent> = fetch_all(
            http,
            token,
            url,
            &[
                ("singleEvents", "true"),
                ("orderBy", "startTime"),
                ("maxResults", "2500"),
                ("timeMin", time_min),
                ("timeMax", time_max),
            ],
        )
        .await?;
        Ok::<_, Error>(items.into_iter().filter_map(|e| to_event(e, id)).collect::<Vec<_>>())
    });
    Ok(try_join_all(per_calendar).await?.into_iter().flatten().collect())
}

fn to_event(e: ApiEvent, calendar_id: &str) -> Option<Event> {
    if e.status.as_deref() == Some("cancelled") {
        return None;
    }
    let start = e.start?;
    let all_day = start.date_time.is_none();
    let pick = |t: EventTime| t.date_time.or(t.date);
    Some(Event {
        id: e.id,
        calendar_id: calendar_id.to_string(),
        title: e.summary.unwrap_or_else(|| "(No title)".into()),
        start: pick(start)?,
        end: e.end.and_then(pick),
        all_day,
        location: e.location,
        description: e.description,
        html_link: e.html_link,
        meet_link: e.hangout_link,
    })
}
