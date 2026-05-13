//! Shared serde models used across multiple resource bodies.

use serde::{Deserialize, Serialize};

/// A piece "helmet" — a small visual badge attached to a player. Some
/// responses include only `{key, color}` (e.g. on a leaderboard row);
/// other contexts wrap it under a [`PlayerInfo`] field.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Helmet {
    pub key: String,
    pub color: String,
}

/// Compact player identity returned inline on match-history rows and
/// similar two-player contexts.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInfo {
    pub address: String,
    /// Absent for "ghost" wallets — addresses that played games but
    /// never registered a username. The /user/{addr} endpoint also 404s
    /// for these.
    #[serde(default)]
    pub username: Option<String>,
    /// Absent for ghost wallets; see [`Self::username`].
    #[serde(default)]
    pub username_display: Option<String>,
    /// Absent for users who haven't equipped one.
    #[serde(default)]
    pub helmet: Option<Helmet>,
}

/// Marker the server attaches when it wants the client to prompt the
/// caller to sign up / log in.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SuggestSignup {
    pub reason: String,
}

/// Optional response envelope (`_meta`) — currently only carries the
/// signup hint, but future server-side additions land here too.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMeta {
    #[serde(default)]
    pub suggest_signup: Option<SuggestSignup>,
}

/// Serde deserializer for the server's nested `{year, month, day}` date
/// shape, used on a handful of summary endpoints
/// (`/auctions/today-summary.date`, `DailyVolume.date`, etc.).
///
/// Field-level usage:
///
/// ```ignore
/// #[derive(Deserialize)]
/// struct Foo {
///     #[serde(deserialize_with = "pixiechess_client::models::common::date_dict::deserialize")]
///     date: chrono::NaiveDate,
/// }
/// ```
pub mod date_dict {
    use chrono::NaiveDate;
    use serde::{Deserialize, Deserializer};

    #[derive(Deserialize)]
    struct Ymd {
        year: i32,
        month: u32,
        day: u32,
    }

    /// Deserialize a `{year, month, day}` object into [`NaiveDate`].
    ///
    /// # Errors
    ///
    /// Returns a serde error if the object is missing one of the fields
    /// or carries values that don't form a valid Gregorian date.
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<NaiveDate, D::Error> {
        let ymd = Ymd::deserialize(d)?;
        NaiveDate::from_ymd_opt(ymd.year, ymd.month, ymd.day)
            .ok_or_else(|| serde::de::Error::custom("invalid date"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn helmet_round_trip() {
        let h = Helmet {
            key: "knightmare".into(),
            color: "blue".into(),
        };
        let s = serde_json::to_string(&h).unwrap();
        assert!(s.contains("\"key\":\"knightmare\""));
        let back: Helmet = serde_json::from_str(&s).unwrap();
        assert_eq!(back, h);
    }

    #[test]
    fn player_info_full() {
        let raw = json!({
            "address": "0xabc",
            "username": "alice",
            "usernameDisplay": "Alice",
            "helmet": {"key": "knightmare", "color": "red"},
        });
        let p: PlayerInfo = serde_json::from_value(raw).unwrap();
        assert_eq!(p.address, "0xabc");
        assert_eq!(p.username.as_deref(), Some("alice"));
        assert_eq!(p.username_display.as_deref(), Some("Alice"));
        assert_eq!(p.helmet.as_ref().unwrap().key, "knightmare");
    }

    #[test]
    fn player_info_decodes_ghost_address() {
        // Ghost addresses (played games but never registered) appear in
        // match-history payloads with only `address` populated.
        let raw = json!({"address": "0xdef"});
        let p: PlayerInfo = serde_json::from_value(raw).unwrap();
        assert_eq!(p.address, "0xdef");
        assert!(p.username.is_none());
        assert!(p.username_display.is_none());
        assert!(p.helmet.is_none());
    }

    #[test]
    fn player_info_missing_address_errors() {
        let raw = json!({"username": "alice"});
        let res: Result<PlayerInfo, _> = serde_json::from_value(raw);
        assert!(res.is_err());
    }

    #[test]
    fn suggest_signup_round_trip() {
        let raw = json!({"reason": "guest_view"});
        let s: SuggestSignup = serde_json::from_value(raw).unwrap();
        assert_eq!(s.reason, "guest_view");
    }

    #[test]
    fn response_meta_empty_object() {
        let raw = json!({});
        let m: ResponseMeta = serde_json::from_value(raw).unwrap();
        assert!(m.suggest_signup.is_none());
    }

    #[test]
    fn response_meta_with_signup_hint() {
        let raw = json!({"suggestSignup": {"reason": "tournament_view"}});
        let m: ResponseMeta = serde_json::from_value(raw).unwrap();
        assert_eq!(m.suggest_signup.unwrap().reason, "tournament_view");
    }

    // date_dict deserializer ---------------------------------------------

    #[derive(serde::Deserialize)]
    struct WithDate {
        #[serde(deserialize_with = "super::date_dict::deserialize")]
        date: chrono::NaiveDate,
    }

    #[test]
    fn date_dict_parses_valid_ymd() {
        let raw = json!({"date": {"year": 2026, "month": 5, "day": 13}});
        let wd: WithDate = serde_json::from_value(raw).unwrap();
        assert_eq!(
            wd.date,
            chrono::NaiveDate::from_ymd_opt(2026, 5, 13).unwrap()
        );
    }

    #[test]
    fn date_dict_rejects_invalid_ymd() {
        let raw = json!({"date": {"year": 2026, "month": 13, "day": 99}});
        let res: Result<WithDate, _> = serde_json::from_value(raw);
        assert!(res.is_err());
    }

    #[test]
    fn date_dict_rejects_missing_field() {
        let raw = json!({"date": {"year": 2026, "month": 5}});
        let res: Result<WithDate, _> = serde_json::from_value(raw);
        assert!(res.is_err());
    }
}
