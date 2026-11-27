//! The text of the terms and the privacy policy.

use topcoat::context::Cx;

use crate::l10n::t;

/// What the texts mention: the instance and its rules.
pub(crate) struct DocCtx {
    pub(crate) name: String,
    pub(crate) domain: String,
    pub(crate) min_age: u8,
    pub(crate) rules: Vec<String>,
}

/// One piece of a section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Block {
    P(String),
    Ul(Vec<String>),
    Ol(Vec<String>),
    /// A lead-in sentence followed by the instance contact.
    Contact(String),
}

/// A headed part of a document; `id` is its anchor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Section {
    pub(crate) id: &'static str,
    pub(crate) title: String,
    pub(crate) blocks: Vec<Block>,
}

macro_rules! s {
    ($cx:expr, $($message:tt)*) => {
        t!($($message)*).resolve($cx).into_owned()
    };
}

fn section(id: &'static str, title: String, blocks: Vec<Block>) -> Section {
    Section { id, title, blocks }
}

fn p(text: String) -> Block {
    Block::P(text)
}

/// The terms of service.
#[must_use]
pub(crate) fn terms_sections(cx: &Cx, ctx: &DocCtx) -> Vec<Section> {
    let (name, domain) = (ctx.name.as_str(), ctx.domain.as_str());
    let mut rules = vec![p(s!(cx, docs_terms_rules_lead, name: name))];
    if !ctx.rules.is_empty() {
        rules.push(Block::Ol(ctx.rules.clone()));
    }
    rules.push(p(s!(cx, docs_terms_rules_p)));
    vec![
        section(
            "agreement",
            s!(cx, docs_terms_agreement_h),
            vec![p(
                s!(cx, docs_terms_agreement_p, name: name, domain: domain),
            )],
        ),
        section(
            "eligibility",
            s!(cx, docs_terms_eligibility_h),
            vec![p(
                s!(cx, docs_terms_eligibility_p, minAge: i32::from(ctx.min_age)),
            )],
        ),
        section(
            "account",
            s!(cx, docs_terms_account_h),
            vec![Block::Ul(vec![
                s!(cx, docs_terms_account_1),
                s!(cx, docs_terms_account_2),
                s!(cx, docs_terms_account_3),
            ])],
        ),
        section(
            "content",
            s!(cx, docs_terms_content_h),
            vec![
                p(s!(cx, docs_terms_content_p1)),
                p(s!(cx, docs_terms_content_p2)),
            ],
        ),
        section("rules", s!(cx, docs_terms_rules_h), rules),
        section(
            "federation",
            s!(cx, docs_terms_federation_h),
            vec![p(s!(cx, docs_terms_federation_p, name: name))],
        ),
        section(
            "moderation",
            s!(cx, docs_terms_moderation_h),
            vec![p(s!(cx, docs_terms_moderation_p))],
        ),
        section(
            "termination",
            s!(cx, docs_terms_termination_h),
            vec![p(s!(cx, docs_terms_termination_p))],
        ),
        section(
            "disclaimers",
            s!(cx, docs_terms_disclaimers_h),
            vec![p(s!(cx, docs_terms_disclaimers_p))],
        ),
        section(
            "changes",
            s!(cx, docs_terms_changes_h),
            vec![p(s!(cx, docs_terms_changes_p))],
        ),
        section(
            "contact",
            s!(cx, docs_terms_contact_h),
            vec![Block::Contact(s!(cx, docs_terms_contact_lead))],
        ),
    ]
}

/// The privacy policy.
#[must_use]
pub(crate) fn privacy_sections(cx: &Cx, ctx: &DocCtx) -> Vec<Section> {
    let (name, domain) = (ctx.name.as_str(), ctx.domain.as_str());
    vec![
        section(
            "summary",
            s!(cx, docs_privacy_summary_h),
            vec![Block::Ul(vec![
                s!(cx, docs_privacy_summary_1),
                s!(cx, docs_privacy_summary_2),
                s!(cx, docs_privacy_summary_3),
                s!(cx, docs_privacy_summary_4),
                s!(cx, docs_privacy_summary_5),
            ])],
        ),
        section(
            "controller",
            s!(cx, docs_privacy_controller_h),
            vec![p(
                s!(cx, docs_privacy_controller_p, name: name, domain: domain),
            )],
        ),
        section(
            "collect",
            s!(cx, docs_privacy_collect_h),
            vec![
                Block::Ul(vec![
                    s!(cx, docs_privacy_collect_1),
                    s!(cx, docs_privacy_collect_2),
                    s!(cx, docs_privacy_collect_3),
                    s!(cx, docs_privacy_collect_4),
                    s!(cx, docs_privacy_collect_5),
                ]),
                p(s!(cx, docs_privacy_collect_p)),
            ],
        ),
        section(
            "use",
            s!(cx, docs_privacy_use_h),
            vec![p(s!(cx, docs_privacy_use_p))],
        ),
        section(
            "visibility",
            s!(cx, docs_privacy_visibility_h),
            vec![p(s!(cx, docs_privacy_visibility_p))],
        ),
        section(
            "federation",
            s!(cx, docs_privacy_federation_h),
            vec![p(s!(cx, docs_privacy_federation_p))],
        ),
        section(
            "cookies",
            s!(cx, docs_privacy_cookies_h),
            vec![p(s!(cx, docs_privacy_cookies_p))],
        ),
        section(
            "retention",
            s!(cx, docs_privacy_retention_h),
            vec![Block::Ul(vec![
                s!(cx, docs_privacy_retention_1),
                s!(cx, docs_privacy_retention_2),
                s!(cx, docs_privacy_retention_3),
                s!(cx, docs_privacy_retention_4),
                s!(cx, docs_privacy_retention_5),
            ])],
        ),
        section(
            "rights",
            s!(cx, docs_privacy_rights_h),
            vec![
                Block::Contact(s!(cx, docs_privacy_rights_p)),
                p(s!(cx, docs_privacy_rights_end)),
            ],
        ),
        section(
            "security",
            s!(cx, docs_privacy_security_h),
            vec![p(s!(cx, docs_privacy_security_p))],
        ),
        section(
            "children",
            s!(cx, docs_privacy_children_h),
            vec![p(s!(cx, docs_privacy_children_p))],
        ),
        section(
            "changes",
            s!(cx, docs_privacy_changes_h),
            vec![p(s!(cx, docs_privacy_changes_p))],
        ),
        section(
            "contact",
            s!(cx, docs_privacy_contact_h),
            vec![Block::Contact(s!(cx, docs_privacy_contact_lead))],
        ),
    ]
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::context;

    fn ctx(rules: &[&str]) -> DocCtx {
        DocCtx {
            name: "Tuita Test".to_owned(),
            domain: "tuita.test".to_owned(),
            min_age: 16,
            rules: rules.iter().map(|rule| (*rule).to_owned()).collect(),
        }
    }

    #[test]
    fn terms_name_the_instance_and_the_minimum_age() {
        let cx = context("en");
        let terms = terms_sections(&cx, &ctx(&[]));
        let ids: Vec<_> = terms.iter().map(|section| section.id).collect();
        assert_eq!(
            ids,
            [
                "agreement",
                "eligibility",
                "account",
                "content",
                "rules",
                "federation",
                "moderation",
                "termination",
                "disclaimers",
                "changes",
                "contact"
            ]
        );
        let Block::P(agreement) = &terms[0].blocks[0] else {
            panic!("agreement is a paragraph");
        };
        assert!(agreement.contains("Tuita Test") && agreement.contains("tuita.test"));
        let Block::P(age) = &terms[1].blocks[0] else {
            panic!("eligibility is a paragraph");
        };
        assert!(age.contains("16"), "{age}");
        assert!(matches!(terms[10].blocks[0], Block::Contact(_)));
    }

    #[test]
    fn the_instance_rules_become_a_numbered_list_only_when_there_are_some() {
        let cx = context("en");
        let none = terms_sections(&cx, &ctx(&[]));
        assert_eq!(none[4].blocks.len(), 2);
        let some = terms_sections(&cx, &ctx(&["Be kind", "No spam"]));
        assert_eq!(some[4].blocks.len(), 3);
        assert_eq!(
            some[4].blocks[1],
            Block::Ol(vec!["Be kind".to_owned(), "No spam".to_owned()])
        );
    }

    #[test]
    fn privacy_has_its_thirteen_sections_and_ends_with_the_contact() {
        let cx = context("en");
        let privacy = privacy_sections(&cx, &ctx(&[]));
        assert_eq!(privacy.len(), 13);
        assert_eq!(privacy[0].id, "summary");
        assert!(matches!(&privacy[0].blocks[0], Block::Ul(items) if items.len() == 5));
        assert_eq!(privacy[12].id, "contact");
        assert!(matches!(privacy[12].blocks[0], Block::Contact(_)));
        assert!(matches!(privacy[8].blocks[0], Block::Contact(_)));
    }

    #[test]
    fn the_texts_come_in_the_visitors_language() {
        let en = terms_sections(&context("en"), &ctx(&[]));
        let pl = terms_sections(&context("pl"), &ctx(&[]));
        assert_ne!(en[0].title, pl[0].title);
    }
}
