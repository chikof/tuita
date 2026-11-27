//! The key other instances check this instance's requests against.

use topcoat::{
    Result,
    context::Cx,
    runtime::Event,
    view::{View, attributes, component, view},
};
use tuita_server::dto::InstanceActorDto;

use crate::{
    admin::{common::tr, federation::group_hex},
    icons::Icon,
    ui::{
        button::ButtonSize,
        icon_button::{IconButtonVariant, icon_button},
    },
};

/// The actor id, the key id and the fingerprint in groups of eight, with a button that copies it.
#[component]
pub(crate) async fn actor_key(cx: &Cx, actor: &InstanceActorDto) -> Result<impl View> {
    let grouped = group_hex(&actor.fingerprint, 8);
    let fingerprint = actor.fingerprint.clone();
    let copy_label = tr!(cx, admin_fed_actor_copy);
    let id_label = tr!(cx, admin_fed_actor_id);
    let key_label = tr!(cx, admin_fed_actor_key);
    let print_label = tr!(cx, admin_fed_actor_fingerprint);
    Ok(view! {
        <dl class="facts" data-c="admin-actor-key">
            <div>
                <dt>(&id_label)</dt>
                <dd><code>(&actor.id)</code></dd>
            </div>
            <div>
                <dt>(&key_label)</dt>
                <dd><code>(&actor.key_id)</code></dd>
            </div>
            <div>
                <dt>(&print_label)</dt>
                <dd class="print">
                    <code data-fingerprint=(&actor.fingerprint)>(&grouped)</code>
                    icon_button(
                        glyph: Icon::Copy,
                        variant: IconButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        label: &copy_label,
                        attrs: attributes! {
                            @click=$(|_e: Event| raw!(
                                    "document.dispatchEvent(new CustomEvent('tuita-copy',{detail:{text:''+${fingerprint},ok:'admin_fed_actor_copied',fail:'admin_fed_actor_copy_failed',v:'fingerprint'}}))",
                                    fingerprint.is_empty(),
                                ))
                        }
                    )
                </dd>
            </div>
        </dl>
    })
}
