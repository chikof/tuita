//! The delivery queue: how much waits, what fails, and a button to retry.

use topcoat::{
    Result,
    context::Cx,
    runtime::{Event, Signal, procedure, signal},
    view::{View, attributes, component, view},
};
use tuita_server::dto::{FederationQueueDto, RetryDto};

use crate::{
    admin::common::{message, stamp, tr},
    api,
    fmt::{compact, full_time},
    l10n::t,
    ui::button::{ButtonVariant, button},
};

/// `POST /admin/federation/retry`. Answers the toast text.
#[procedure]
pub(crate) async fn retry_deliveries(cx: &Cx) -> Result<std::result::Result<String, String>> {
    Ok(
        api::post::<_, RetryDto>(cx, "/admin/federation/retry", &serde_json::json!({}))
            .await
            .map(|done| {
                tr!(
                    cx,
                    admin_fed_retry_done,
                    count: done.requeued,
                    compact: compact(cx, i64::from(done.requeued))
                )
            })
            .map_err(message),
    )
}

/// One failing instance with the words of its line.
struct Failing {
    domain: String,
    line: String,
    error: Option<String>,
}

/// The numbers of `queue` and the instances that fail. `revision` is bumped after a retry so the
/// shard around the summary renders again.
#[component]
pub(crate) async fn queue_summary(
    cx: &Cx,
    queue: &FederationQueueDto,
    revision: &Signal<u32>,
) -> Result<impl View> {
    let busy = signal(cx, || false);
    let waiting = tr!(
        cx,
        admin_fed_queue_waiting,
        count: queue.count,
        compact: compact(cx, i64::from(queue.count))
    );
    let mut parts = vec![tr!(cx, admin_fed_queue_due, count: queue.due)];
    if let Some(at) = queue.oldest {
        parts.push(tr!(cx, admin_fed_queue_oldest, date: full_time(cx, &stamp(at))));
    }
    let summary = parts.join(" · ");
    let failing: Vec<Failing> = queue
        .failing
        .iter()
        .map(|row| Failing {
            domain: row.domain.clone(),
            line: format!(
                "{} · {} · {}",
                tr!(
                    cx,
                    admin_fed_failing_count,
                    count: row.count,
                    compact: compact(cx, i64::from(row.count))
                ),
                tr!(
                    cx,
                    admin_fed_failing_attempts,
                    count: row.attempts,
                    compact: compact(cx, i64::from(row.attempts))
                ),
                tr!(cx, admin_fed_failing_next, date: full_time(cx, &stamp(row.next_attempt_at)))
            ),
            error: row.last_error.clone(),
        })
        .collect();
    let empty = queue.count == 0;
    let count = queue.count.to_string();
    Ok(view! {
        <div class="queue" data-c="admin-queue-summary">
            <div class="head">
                <div class="numbers">
                    if empty {
                        <p>(t!(admin_fed_queue_empty))</p>
                    } else {
                        <p class="count" data-queue-count=(&count)>(&waiting)</p>
                        <p class="dim">(&summary)</p>
                    }
                </div>
                button(
                    variant: ButtonVariant::Outline,
                    attrs: attributes! {
                        :disabled=$(if empty { true } else { busy.get() })
                        @click=$(async |_e: Event| {
                            busy.set(true);
                            let done = retry_deliveries().await;
                            busy.set(false);
                            if done.is_ok() {
                                let msg = done.unwrap();
                                raw!(
                                    "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:''+${msg}}));",
                                    drop(msg),
                                );
                                revision.increment();
                                false
                            } else {
                                let msg = done.unwrap_err();
                                raw!(
                                    "document.dispatchEvent(new CustomEvent('tuita-toast',{bubbles:true,detail:''+${msg}}))",
                                    msg.is_empty(),
                                )
                            }
                        })
                    },
                    (t!(admin_fed_retry))
                )
            </div>
            <h3>(t!(admin_fed_failing))</h3>
            if failing.is_empty() {
                <p class="dim">(t!(admin_fed_failing_none))</p>
            } else {
                <ul class="failing">
                    for row in &failing {
                        <li data-failing=(&row.domain)>
                            <p class="line">
                                <code class="domain">(&row.domain)</code>
                                <span class="dim">(&row.line)</span>
                            </p>
                            <p class="error">
                                <span class="label">(t!(admin_fed_failing_error))</span>
                                if let Some(text) = &row.error {
                                    <code>(text)</code>
                                } else {
                                    <span class="dim">(t!(admin_fed_failing_no_error))</span>
                                }
                            </p>
                        </li>
                    }
                </ul>
            }
        </div>
    })
}
