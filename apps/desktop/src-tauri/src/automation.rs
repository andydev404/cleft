use crate::classifier::ContentType;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

// Applies to enabled rules only — disabled rules can be stored without
// limit, they just don't run.
pub const MAX_ACTIVE_RULES: i64 = 10;

// Deserialize is for the IPC boundary (frontend sends one of these as a
// create_rule argument); as_str/from_str below are the separate DB-storage
// round-trip, since the column is plain TEXT, not JSON.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerKind {
    AppIs,
    UrlContains,
    ContentTypeIs,
    WindowTitleContains,
    ContentContains,
}

impl TriggerKind {
    fn as_str(&self) -> &'static str {
        match self {
            TriggerKind::AppIs => "AppIs",
            TriggerKind::UrlContains => "UrlContains",
            TriggerKind::ContentTypeIs => "ContentTypeIs",
            TriggerKind::WindowTitleContains => "WindowTitleContains",
            TriggerKind::ContentContains => "ContentContains",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "AppIs" => TriggerKind::AppIs,
            "UrlContains" => TriggerKind::UrlContains,
            "ContentTypeIs" => TriggerKind::ContentTypeIs,
            "WindowTitleContains" => TriggerKind::WindowTitleContains,
            "ContentContains" => TriggerKind::ContentContains,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionKind {
    AssignCollection,
    AddTag,
    Pin,
    Block,
    AssignWorkspace,
}

impl ActionKind {
    fn as_str(&self) -> &'static str {
        match self {
            ActionKind::AssignCollection => "AssignCollection",
            ActionKind::AddTag => "AddTag",
            ActionKind::Pin => "Pin",
            ActionKind::Block => "Block",
            ActionKind::AssignWorkspace => "AssignWorkspace",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "AssignCollection" => ActionKind::AssignCollection,
            "AddTag" => ActionKind::AddTag,
            "Pin" => ActionKind::Pin,
            "Block" => ActionKind::Block,
            "AssignWorkspace" => ActionKind::AssignWorkspace,
            _ => return None,
        })
    }
}

#[derive(Serialize, Clone)]
pub struct Rule {
    pub id: String,
    pub trigger_kind: TriggerKind,
    pub trigger_value: String,
    pub action_kind: ActionKind,
    pub action_value: String,
    pub enabled: bool,
}

pub fn init_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS automation_rules (
            id TEXT PRIMARY KEY,
            trigger_kind TEXT NOT NULL,
            trigger_value TEXT NOT NULL,
            action_kind TEXT NOT NULL,
            action_value TEXT NOT NULL DEFAULT '',
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL
        )",
        (),
    )?;

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM automation_rules", [], |r| r.get(0))?;
    if count == 0 {
        seed_defaults(conn)?;
    }
    Ok(())
}

// Two example rules so Automation isn't an empty list on first launch.
fn seed_defaults(conn: &Connection) -> rusqlite::Result<()> {
    insert_rule(
        conn,
        TriggerKind::ContentTypeIs,
        ContentType::Color.as_str(),
        ActionKind::AssignCollection,
        "Design / Brand Colors",
    )?;
    insert_rule(
        conn,
        TriggerKind::UrlContains,
        "github.com",
        ActionKind::AddTag,
        "dev",
    )?;
    Ok(())
}

fn insert_rule(
    conn: &Connection,
    trigger_kind: TriggerKind,
    trigger_value: &str,
    action_kind: ActionKind,
    action_value: &str,
) -> rusqlite::Result<Rule> {
    let id = Uuid::new_v4().to_string();
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    conn.execute(
        "INSERT INTO automation_rules (id, trigger_kind, trigger_value, action_kind, action_value, enabled, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
        (&id, trigger_kind.as_str(), trigger_value, action_kind.as_str(), action_value, created_at),
    )?;
    Ok(Rule {
        id,
        trigger_kind,
        trigger_value: trigger_value.to_string(),
        action_kind,
        action_value: action_value.to_string(),
        enabled: true,
    })
}

fn active_count(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM automation_rules WHERE enabled = 1",
        [],
        |r| r.get(0),
    )
}

// New rules start enabled, so creation is subject to the same active-rule
// cap as explicitly re-enabling one.
pub fn create_rule(
    conn: &Connection,
    trigger_kind: TriggerKind,
    trigger_value: &str,
    action_kind: ActionKind,
    action_value: &str,
) -> rusqlite::Result<Result<Rule, String>> {
    if active_count(conn)? >= MAX_ACTIVE_RULES {
        return Ok(Err(format!(
            "Maximum {MAX_ACTIVE_RULES} active rules — disable one first"
        )));
    }
    insert_rule(conn, trigger_kind, trigger_value, action_kind, action_value).map(Ok)
}

pub fn set_enabled(
    conn: &Connection,
    id: &str,
    enabled: bool,
) -> rusqlite::Result<Result<(), String>> {
    if enabled && active_count(conn)? >= MAX_ACTIVE_RULES {
        return Ok(Err(format!(
            "Maximum {MAX_ACTIVE_RULES} active rules — disable one first"
        )));
    }
    conn.execute(
        "UPDATE automation_rules SET enabled = ?1 WHERE id = ?2",
        (enabled, id),
    )?;
    Ok(Ok(()))
}

// Editing never has to touch the active-rule cap — it doesn't change
// enabled state, just what an already-counted rule matches/does.
pub fn update_rule(
    conn: &Connection,
    id: &str,
    trigger_kind: TriggerKind,
    trigger_value: &str,
    action_kind: ActionKind,
    action_value: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE automation_rules SET trigger_kind = ?1, trigger_value = ?2, action_kind = ?3, action_value = ?4 WHERE id = ?5",
        (trigger_kind.as_str(), trigger_value, action_kind.as_str(), action_value, id),
    )?;
    Ok(())
}

pub fn delete_rule(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM automation_rules WHERE id = ?1", [id])?;
    Ok(())
}

pub fn list_rules(conn: &Connection) -> rusqlite::Result<Vec<Rule>> {
    let mut stmt = conn.prepare(
        "SELECT id, trigger_kind, trigger_value, action_kind, action_value, enabled
         FROM automation_rules ORDER BY created_at",
    )?;
    let rows = stmt.query_map([], |r| {
        let trigger_kind: String = r.get(1)?;
        let action_kind: String = r.get(3)?;
        Ok(Rule {
            id: r.get(0)?,
            trigger_kind: TriggerKind::from_str(&trigger_kind)
                .unwrap_or(TriggerKind::ContentContains),
            trigger_value: r.get(2)?,
            action_kind: ActionKind::from_str(&action_kind).unwrap_or(ActionKind::AddTag),
            action_value: r.get(4)?,
            enabled: r.get(5)?,
        })
    })?;
    rows.collect()
}

pub struct CaptureContext<'a> {
    pub bundle_id: &'a str,
    pub content: &'a str,
    pub content_type: ContentType,
    pub window_title: Option<&'a str>,
    pub url: Option<&'a str>,
}

#[derive(Default)]
pub struct RuleOutcome {
    pub block: bool,
    pub collection: Option<String>,
    pub tags: Vec<String>,
    pub pin: bool,
    pub workspace: Option<String>,
}

// Only these two trigger kinds need window title / URL — which normally
// arrive asynchronously well after the clip is already saved (see
// context.rs). Callers use this to decide whether a rule-evaluation pass is
// worth a bounded synchronous Accessibility fetch before saving at all.
pub fn needs_window_context(rules: &[Rule]) -> bool {
    rules.iter().any(|r| {
        r.enabled
            && matches!(
                r.trigger_kind,
                TriggerKind::WindowTitleContains | TriggerKind::UrlContains
            )
    })
}

fn contains_ci(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

fn trigger_matches(rule: &Rule, ctx: &CaptureContext) -> bool {
    match rule.trigger_kind {
        TriggerKind::AppIs => ctx.bundle_id.eq_ignore_ascii_case(&rule.trigger_value),
        TriggerKind::UrlContains => ctx.url.is_some_and(|u| contains_ci(u, &rule.trigger_value)),
        TriggerKind::ContentTypeIs => ctx
            .content_type
            .as_str()
            .eq_ignore_ascii_case(&rule.trigger_value),
        TriggerKind::WindowTitleContains => ctx
            .window_title
            .is_some_and(|t| contains_ci(t, &rule.trigger_value)),
        TriggerKind::ContentContains => contains_ci(ctx.content, &rule.trigger_value),
    }
}

// Runs at capture time, in the Rust core, before the clip is emitted to the
// frontend. A matching Block action short-circuits immediately
// — it takes precedence over anything else at that point since there's no
// clip left to apply other actions to. Otherwise every matching enabled
// rule's action accumulates onto one outcome.
pub fn evaluate(rules: &[Rule], ctx: &CaptureContext) -> RuleOutcome {
    let mut outcome = RuleOutcome::default();
    for rule in rules.iter().filter(|r| r.enabled) {
        if !trigger_matches(rule, ctx) {
            continue;
        }
        match rule.action_kind {
            ActionKind::Block => {
                outcome.block = true;
                return outcome;
            }
            ActionKind::AssignCollection => outcome.collection = Some(rule.action_value.clone()),
            ActionKind::AddTag => outcome.tags.push(rule.action_value.clone()),
            ActionKind::Pin => outcome.pin = true,
            ActionKind::AssignWorkspace => outcome.workspace = Some(rule.action_value.clone()),
        }
    }
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx<'a>(
        bundle_id: &'a str,
        content: &'a str,
        content_type: ContentType,
        window_title: Option<&'a str>,
        url: Option<&'a str>,
    ) -> CaptureContext<'a> {
        CaptureContext {
            bundle_id,
            content,
            content_type,
            window_title,
            url,
        }
    }

    fn rule(
        trigger_kind: TriggerKind,
        trigger_value: &str,
        action_kind: ActionKind,
        action_value: &str,
    ) -> Rule {
        Rule {
            id: "r1".into(),
            trigger_kind,
            trigger_value: trigger_value.into(),
            action_kind,
            action_value: action_value.into(),
            enabled: true,
        }
    }

    #[test]
    fn app_is_matches_exact_bundle_id() {
        let rules = vec![rule(
            TriggerKind::AppIs,
            "com.tinyapp.TablePlus",
            ActionKind::AssignWorkspace,
            "Work",
        )];
        let outcome = evaluate(
            &rules,
            &ctx(
                "com.tinyapp.TablePlus",
                "SELECT 1",
                ContentType::Sql,
                None,
                None,
            ),
        );
        assert_eq!(outcome.workspace, Some("Work".to_string()));

        let outcome = evaluate(
            &rules,
            &ctx("com.other.app", "SELECT 1", ContentType::Sql, None, None),
        );
        assert_eq!(outcome.workspace, None);
    }

    #[test]
    fn content_type_is_matches() {
        let rules = vec![rule(
            TriggerKind::ContentTypeIs,
            "Color",
            ActionKind::AssignCollection,
            "Design / Brand Colors",
        )];
        let outcome = evaluate(
            &rules,
            &ctx("app", "#2563EB", ContentType::Color, None, None),
        );
        assert_eq!(
            outcome.collection,
            Some("Design / Brand Colors".to_string())
        );
    }

    #[test]
    fn url_contains_is_case_insensitive() {
        let rules = vec![rule(
            TriggerKind::UrlContains,
            "github.com",
            ActionKind::AddTag,
            "dev",
        )];
        let outcome = evaluate(
            &rules,
            &ctx(
                "app",
                "text",
                ContentType::PlainText,
                None,
                Some("https://GitHub.com/org/repo"),
            ),
        );
        assert_eq!(outcome.tags, vec!["dev"]);
    }

    #[test]
    fn window_title_contains_matches() {
        let rules = vec![rule(
            TriggerKind::WindowTitleContains,
            "prod",
            ActionKind::AddTag,
            "production",
        )];
        let outcome = evaluate(
            &rules,
            &ctx(
                "app",
                "text",
                ContentType::PlainText,
                Some("prod-db — TablePlus"),
                None,
            ),
        );
        assert_eq!(outcome.tags, vec!["production"]);
    }

    #[test]
    fn content_contains_matches() {
        let rules = vec![rule(
            TriggerKind::ContentContains,
            "password",
            ActionKind::Block,
            "",
        )];
        let outcome = evaluate(
            &rules,
            &ctx(
                "app",
                "my password is hunter2",
                ContentType::PlainText,
                None,
                None,
            ),
        );
        assert!(outcome.block);
    }

    #[test]
    fn block_short_circuits_other_actions() {
        let rules = vec![
            rule(TriggerKind::ContentTypeIs, "Color", ActionKind::Block, ""),
            rule(
                TriggerKind::ContentTypeIs,
                "Color",
                ActionKind::AddTag,
                "should-not-apply",
            ),
        ];
        let outcome = evaluate(&rules, &ctx("app", "#000", ContentType::Color, None, None));
        assert!(outcome.block);
        assert!(outcome.tags.is_empty());
    }

    #[test]
    fn disabled_rules_never_match() {
        let mut r = rule(TriggerKind::ContentTypeIs, "Color", ActionKind::Pin, "");
        r.enabled = false;
        let outcome = evaluate(&[r], &ctx("app", "#000", ContentType::Color, None, None));
        assert!(!outcome.pin);
    }

    #[test]
    fn multiple_matching_rules_accumulate_actions() {
        let rules = vec![
            rule(
                TriggerKind::ContentTypeIs,
                "Sql",
                ActionKind::AssignCollection,
                "Work / SQL Queries",
            ),
            rule(
                TriggerKind::WindowTitleContains,
                "prod",
                ActionKind::AddTag,
                "production",
            ),
            rule(TriggerKind::ContentTypeIs, "Sql", ActionKind::Pin, ""),
        ];
        let outcome = evaluate(
            &rules,
            &ctx("app", "SELECT 1", ContentType::Sql, Some("prod-db"), None),
        );
        assert_eq!(outcome.collection, Some("Work / SQL Queries".to_string()));
        assert_eq!(outcome.tags, vec!["production"]);
        assert!(outcome.pin);
    }

    #[test]
    fn needs_window_context_checks_enabled_rules_only() {
        let mut r = rule(
            TriggerKind::WindowTitleContains,
            "prod",
            ActionKind::AddTag,
            "x",
        );
        assert!(needs_window_context(&[r.clone()]));
        r.enabled = false;
        assert!(!needs_window_context(&[r]));
        assert!(!needs_window_context(&[rule(
            TriggerKind::AppIs,
            "com.x",
            ActionKind::AddTag,
            "x"
        )]));
    }

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_table(&conn).unwrap();
        conn
    }

    #[test]
    fn init_seeds_default_rules_once() {
        let conn = test_conn();
        assert_eq!(list_rules(&conn).unwrap().len(), 2);
    }

    #[test]
    fn create_rule_enforces_active_cap() {
        let conn = Connection::open_in_memory().unwrap();
        init_table(&conn).unwrap(); // 2 seeded

        for i in 0..8 {
            let result = create_rule(
                &conn,
                TriggerKind::ContentContains,
                &format!("term{i}"),
                ActionKind::AddTag,
                "x",
            )
            .unwrap();
            assert!(result.is_ok());
        }
        // 10 active now (2 seeded + 8 created).
        let result = create_rule(
            &conn,
            TriggerKind::ContentContains,
            "one-too-many",
            ActionKind::AddTag,
            "x",
        )
        .unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn set_enabled_respects_active_cap() {
        let conn = Connection::open_in_memory().unwrap();
        init_table(&conn).unwrap();
        let rules = list_rules(&conn).unwrap();
        set_enabled(&conn, &rules[0].id, false).unwrap().unwrap();

        for i in 0..9 {
            create_rule(
                &conn,
                TriggerKind::ContentContains,
                &format!("t{i}"),
                ActionKind::AddTag,
                "x",
            )
            .unwrap()
            .unwrap();
        }
        // 9 created + 1 remaining seeded = 10 active; re-enabling the 10th should fail.
        let result = set_enabled(&conn, &rules[0].id, true).unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn update_rule_changes_trigger_and_action() {
        let conn = test_conn();
        let rules = list_rules(&conn).unwrap();
        let id = rules[0].id.clone();
        update_rule(
            &conn,
            &id,
            TriggerKind::AppIs,
            "com.example.App",
            ActionKind::Pin,
            "",
        )
        .unwrap();

        let updated = list_rules(&conn)
            .unwrap()
            .into_iter()
            .find(|r| r.id == id)
            .unwrap();
        assert_eq!(updated.trigger_kind, TriggerKind::AppIs);
        assert_eq!(updated.trigger_value, "com.example.App");
        assert_eq!(updated.action_kind, ActionKind::Pin);
        assert_eq!(updated.enabled, rules[0].enabled);
    }

    #[test]
    fn delete_rule_removes_it() {
        let conn = test_conn();
        let rules = list_rules(&conn).unwrap();
        delete_rule(&conn, &rules[0].id).unwrap();
        assert_eq!(list_rules(&conn).unwrap().len(), 1);
    }
}
