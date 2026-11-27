//! The federation tab: the instances this one talks to.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal, procedure, shard, signal},
    view::{View, attributes, component, view},
};
use tuita_server::dto::{
    CreateDomainBlockRequest, DomainBlockDto, DomainBlockSeverity, FederationDto,
};

use crate::{
    admin::{
        actor_key::actor_key,
        common::{message, tr},
        domain_block_row::domain_block_row,
        federation::severity_options,
        queue_summary::queue_summary,
        select_field::select_field,
        setting_card::setting_card,
    },
    api,
    l10n::t,
    ui::{
        bento::bento, button::button, number_format::NumberFormat, stat_card::stat_card,
        text_field::text_field,
    },
};

/// `POST /admin/federation/blocks`. Answers the toast text.
#[procedure]
pub(crate) async fn block_domain(
    cx: &Cx,
    domain: String,
    severity: String,
    reason: String,
) -> Result<std::result::Result<String, String>> {
    let body = CreateDomainBlockRequest {
        domain: domain.trim().to_owned(),
        severity: DomainBlockSeverity::parse(&severity).unwrap_or(DomainBlockSeverity::Silence),
        reason: reason.trim().to_owned(),
    };
    Ok(
        api::post::<_, DomainBlockDto>(cx, "/admin/federation/blocks", &body)
            .await
            .map(|saved| tr!(cx, admin_fed_blocked, domain: saved.domain))
            .map_err(message),
    )
}

/// The numbers, the queue and the instance actor, from the API again when `revision` changes.
#[shard]
async fn federation_summary(cx: &Cx, revision: Signal<u32>) -> Result<impl View> {
    let _ = revision.get();
    let data = api::get::<FederationDto>(cx, "/admin/federation").await?;
    let n = |value: i32| u64::try_from(value).unwrap_or(0);
    let domains = tr!(cx, admin_fed_stat_domains);
    let domains_sub = tr!(cx, admin_fed_stat_domains_sub);
    let accounts = tr!(cx, admin_fed_stat_accounts);
    let accounts_sub = tr!(cx, admin_fed_stat_accounts_sub);
    let posts = tr!(cx, admin_fed_stat_posts);
    let posts_sub = tr!(cx, admin_fed_stat_posts_sub);
    let queued = tr!(cx, admin_fed_stat_queue);
    let queued_sub = tr!(cx, admin_fed_stat_queue_sub);
    let queue_title = tr!(cx, admin_fed_queue_title);
    let queue_desc = tr!(cx, admin_fed_queue_desc);
    let actor_title = tr!(cx, admin_fed_actor_title);
    let actor_desc = tr!(cx, admin_fed_actor_desc);
    Ok(view! {
        bento(
            stat_card(
                entrance: 0u32,
                label: &domains,
                value: n(data.known_domains),
                format: NumberFormat::Compact,
                sub: domains_sub.as_str()
            )
            stat_card(
                entrance: 1u32,
                label: &accounts,
                value: n(data.remote_accounts),
                format: NumberFormat::Compact,
                sub: accounts_sub.as_str()
            )
            stat_card(
                entrance: 2u32,
                label: &posts,
                value: n(data.remote_posts),
                format: NumberFormat::Compact,
                sub: posts_sub.as_str()
            )
            stat_card(
                entrance: 3u32,
                label: &queued,
                value: n(data.queue.count),
                format: NumberFormat::Compact,
                sub: queued_sub.as_str()
            )
        )
        setting_card(
            title: &queue_title,
            description: queue_desc.as_str(),
            queue_summary(queue: &data.queue, revision: &revision)
        )
        setting_card(
            title: &actor_title,
            description: actor_desc.as_str(),
            actor_key(actor: &data.actor)
        )
    })
}

/// The blocks, from the API again when `revision` changes.
#[shard]
async fn federation_blocks(cx: &Cx, revision: Signal<u32>) -> Result<impl View> {
    let _ = revision.get();
    let data = api::get::<FederationDto>(cx, "/admin/federation").await?;
    let list = tr!(cx, admin_fed_list);
    Ok(view! {
        <h3>(&list)</h3>
        if data.blocks.is_empty() {
            <p class="dim">(t!(admin_fed_none))</p>
        } else {
            <div class="blocks">
                #[key(block.domain.as_bytes())]
                for block in &data.blocks {
                    domain_block_row(block: block, revision: &revision)
                }
            </div>
        }
    })
}

#[component]
pub(crate) async fn federation_tab(cx: &Cx) -> Result<impl View> {
    let domain = signal(cx, String::new);
    let severity = signal(cx, || "silence".to_owned());
    let reason = signal(cx, String::new);
    let error = signal(cx, String::new);
    let busy = signal(cx, || false);
    let revision = signal(cx, || 0u32);
    let options: Vec<(String, String)> = severity_options(cx)
        .into_iter()
        .map(|(value, label)| (value.to_owned(), label))
        .collect();
    let title = tr!(cx, admin_fed_blocks_title);
    let desc = tr!(cx, admin_fed_blocks_desc);
    let domain_label = tr!(cx, admin_fed_domain);
    let domain_hint = tr!(cx, admin_fed_domain_hint);
    let severity_label = tr!(cx, admin_fed_severity);
    let reason_label = tr!(cx, admin_fed_reason);
    let reason_hint = tr!(cx, admin_fed_reason_hint);
    Ok(view! {
        <div class="tab" data-c="admin-federation">
            federation_summary(revision: revision.clone())
            setting_card(
                title: &title,
                description: desc.as_str(),
                <form
                    class="form"
                    novalidate=""
                    @submit=$(async |e: Event| {
                        e.prevent_default();
                        error.set("".to_owned());
                        busy.set(true);
                        let done = block_domain(
                            domain.get(),
                            severity.get(),
                            reason.get(),
                        ).await;
                        busy.set(false);
                        if done.is_ok() {
                            let msg = done.unwrap();
                            domain.set("".to_owned());
                            reason.set("".to_owned());
                            raw!(
                                "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:''+${msg}}));",
                                drop(msg),
                            );
                            revision.increment();
                            false
                        } else {
                            error.set(done.unwrap_err());
                            false
                        }
                    })
                >
                    text_field(
                        label: &domain_label,
                        hint: domain_hint.as_str(),
                        attrs: attributes! {
                            name="domain"
                            autocomplete="off"
                            autocapitalize="none"
                            spellcheck="false"
                            :value=$(domain.get())
                            @input=$(|e: Event| domain.set(e.target.value))
                        }
                    )
                    <div class="pair">
                        select_field(
                            label: &severity_label,
                            options: &options,
                            value: "silence",
                            attrs: attributes! {
                                name="severity"
                                @change=$(|e: Event| severity.set(e.target.value))
                            }
                        )
                        text_field(
                            label: &reason_label,
                            hint: reason_hint.as_str(),
                            attrs: attributes! {
                                name="reason"
                                maxlength="200"
                                :value=$(reason.get())
                                @input=$(|e: Event| reason.set(e.target.value))
                            }
                        )
                    </div>
                    <p class="error" role="alert" :hidden=$(error.get().is_empty())>
                        $(error.get())
                    </p>
                    <div class="end">
                        button(
                            submit: true,
                            attrs: attributes! {
                                :disabled=$(busy.get())
                                :aria-busy=$(if busy.get() { "true" } else { "false" })
                            },
                            (t!(admin_fed_block))
                        )
                    </div>
                </form>
                federation_blocks(revision: revision.clone())
            )
        </div>
    })
}
