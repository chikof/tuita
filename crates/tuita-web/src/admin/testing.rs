//! Fixtures for the render tests of this area.

use time::macros::datetime;
use tuita_server::dto::{AdminReportDto, ReportReason, ReportStatus};
use uuid::Uuid;

use crate::post::testing::{sample_post, sample_user};

/// An open report about a post of `ada`, filed by `bob`.
pub(crate) fn sample_report() -> AdminReportDto {
    AdminReportDto {
        id: Uuid::from_u128(0x7e90),
        reason: ReportReason::Spam,
        note: "an ad".to_owned(),
        status: ReportStatus::Open,
        resolution: None,
        created_at: datetime!(2026-11-09 10:00 UTC),
        resolved_at: None,
        reporter: Some(sample_user("bob")),
        target: sample_user("ada"),
        target_suspended: false,
        resolved_by: None,
        post: Some(sample_post()),
        same_target_open: 1,
    }
}
