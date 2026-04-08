use chrono::{DateTime, Utc};
use cron::Schedule as CronSchedule;
use kittypaw_core::package::SkillPackage;
use kittypaw_core::skill::Skill;
use std::str::FromStr;

/// Validate a cron expression and enforce minimum 5-minute interval.
pub fn validate_cron(expr: &str) -> Result<(), String> {
    let schedule =
        CronSchedule::from_str(expr).map_err(|e| format!("Invalid cron expression: {e}"))?;

    // Check minimum interval: get next 2 occurrences and ensure gap >= 5 min
    let now = Utc::now();
    let mut upcoming = schedule.upcoming(Utc).take(2);
    if let (Some(first), Some(second)) = (upcoming.next(), upcoming.next()) {
        let gap = second - first;
        if gap.num_minutes() < 5 {
            return Err(format!(
                "Schedule interval too short ({} minutes). Minimum is 5 minutes.",
                gap.num_minutes()
            ));
        }
    }
    let _ = now;
    Ok(())
}

/// Check whether a cron expression has fired since the last run.
pub fn is_cron_due(cron_expr: &str, last_run: Option<DateTime<Utc>>) -> bool {
    let schedule = match CronSchedule::from_str(cron_expr) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let reference = last_run.unwrap_or_else(|| Utc::now());
    schedule
        .after(&reference)
        .take_while(|t| *t <= Utc::now())
        .next()
        .is_some()
}

/// Check if a package is due to run based on its cron trigger.
pub fn is_package_due(pkg: &SkillPackage, last_run: Option<DateTime<Utc>>) -> bool {
    let trigger = match &pkg.trigger {
        Some(t) if t.trigger_type == "schedule" => t,
        _ => return false,
    };
    match &trigger.cron {
        Some(c) => is_cron_due(c, last_run),
        None => false,
    }
}

/// Check if a one-shot skill is due to run.
///
/// A once skill is due when:
/// 1. The skill is enabled
/// 2. `run_at` is set and its datetime has passed
/// 3. `last_run` is None (hasn't executed yet)
pub fn is_once_due(skill: &Skill, last_run: Option<DateTime<Utc>>) -> bool {
    if skill.trigger.trigger_type != "once" || !skill.enabled {
        return false;
    }
    if last_run.is_some() {
        return false; // already ran
    }
    match &skill.trigger.run_at {
        Some(run_at_str) => match run_at_str.parse::<DateTime<Utc>>() {
            Ok(run_at) => run_at <= Utc::now(),
            Err(_) => {
                tracing::warn!(
                    name = skill.name.as_str(),
                    run_at = run_at_str.as_str(),
                    "Once skill has invalid run_at timestamp — skipping"
                );
                false
            }
        },
        None => false,
    }
}

/// Check if a skill is due to run based on its trigger.
pub fn is_due(skill: &Skill, last_run: Option<DateTime<Utc>>) -> bool {
    if !skill.enabled {
        return false;
    }
    match skill.trigger.trigger_type.as_str() {
        "schedule" => match &skill.trigger.cron {
            Some(c) => is_cron_due(c, last_run),
            None => false,
        },
        "once" => is_once_due(skill, last_run),
        _ => false,
    }
}
