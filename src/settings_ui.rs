//! Reusable, host-styled DNS provider settings form.
//!
//! The host handles submission, validation, authorization and secure storage.
//! Never pass saved secrets to this component.
/// CSS for provider-specific field visibility. Hosts should include this in their stylesheet.
pub const PROVIDER_FORM_CSS: &str = include_str!("settings_ui.css");
/// Browser behavior for provider-specific TTL selection. Include once on pages using the form.
pub const PROVIDER_FORM_JS: &str = include_str!("settings_ui.js");

use crate::{
    ClouDnsAuthentication,
    components::{button::{button, ButtonVariant}, input::input, label::label, select::select, switch::switch},
};
use topcoat::{
    Result,
    context::Cx,
    icon::{icon, iconify::iconify_icon},
    view::{Attributes, View, attributes, component, view},
};

/// Provider-specific, non-secret values shown in the form.
#[derive(Clone, Debug)]
pub enum ProviderFormProvider {
    Cloudflare,
    Cloudns {
        authentication: ClouDnsAuthentication,
    },
}

/// Non-secret values shown in the provider form.
#[derive(Clone, Debug)]
pub struct ProviderFormValues {
    pub enabled: bool,
    pub provider: ProviderFormProvider,
    pub zone: String,
    /// Cloudflare uses 1 for automatic TTL.
    pub ttl: String,
    /// Whether the selected provider already has a stored credential.
    pub credential_saved: bool,
}

impl Default for ProviderFormValues {
    fn default() -> Self {
        Self {
            enabled: true,
            provider: ProviderFormProvider::Cloudflare,
            zone: String::new(),
            ttl: "3600".into(),
            credential_saved: false,
        }
    }
}

/// DNS provider form. `id` must be unique on the page. The host supplies the
/// form action and method through `attrs`, and re-renders with `busy` while saving.
/// Fields submit `provider`, `zone`, `ttl`, `cloudns_auth_type`,
/// `cloudns_auth_id`, and `credential`. A blank credential means retain the
/// existing secret only if it belongs to the selected provider. Hosts must
/// validate all values and require credentials for a new provider.
///
/// Tailwind semantic colour tokens follow the host's light/dark theme. The
/// forwarded form attributes and `dns-provider-*` classes allow host styling.
#[component]
pub async fn dns_provider_form(
    cx: &Cx,
    id: &str,
    values: &ProviderFormValues,
    #[default] busy: bool,
    #[default] mut attrs: Attributes,
) -> Result<impl View> {
    let provider_id = format!("{id}-provider");
    let enabled_id = format!("{id}-enabled");
    let zone_id = format!("{id}-zone");
    let ttl_id = format!("{id}-ttl");
    let auth_type_id = format!("{id}-auth-type");
    let auth_id = format!("{id}-auth-id");
    let credential_id = format!("{id}-credential");
    let ttl = &values.ttl;
    let ttl_auto = matches!(values.provider, ProviderFormProvider::Cloudflare) && ttl == "1";
    let saved_provider = if values.credential_saved { if matches!(values.provider, ProviderFormProvider::Cloudflare) { "cloudflare" } else { "cloudns" } } else { "" };
    // The controls remain available when switching providers in the browser.
    let (is_cloudns, is_sub_auth, cloudns_auth_id) = match &values.provider {
        ProviderFormProvider::Cloudflare => (false, false, ""),
        ProviderFormProvider::Cloudns { authentication } => match authentication {
            ClouDnsAuthentication::AuthId(id) => (true, false, id.as_str()),
            ClouDnsAuthentication::SubAuthId(id) => (true, true, id.as_str()),
        },
    };
    let button_text = if busy { "Saving…" } else { "Save" };
    let button_icon = if busy {
        iconify_icon!("lucide:loader-circle")
    } else {
        iconify_icon!("lucide:save")
    };
    let icon_class = if busy {
        "size-4 animate-spin"
    } else {
        "size-4"
    };
    Ok(view! {
        <form class=(topcoat::view::class!("dns-provider-form grid gap-4 text-foreground", attrs.remove("class"))) (attrs) aria-busy=(busy) data-saved-provider=(saved_provider)>
            <div class="dns-provider-field grid min-w-0 items-center gap-2 sm:grid-cols-[12rem_minmax(0,26rem)]">
                label(attrs: attributes! { for=(&enabled_id) }, "Enable")
                switch(attrs: attributes! { id=(&enabled_id) name="enabled" value="true" checked=(values.enabled) disabled=(busy) })
            </div>
            <input type="hidden" name="settings_action" value="save" />
            <fieldset class="dns-provider-settings grid min-w-0 gap-4" disabled=(!values.enabled || busy)>
            <div class="dns-provider-field grid min-w-0 items-center gap-2 sm:grid-cols-[12rem_minmax(0,26rem)]">
                label(attrs: attributes! { for=(&provider_id) class="dns-provider-required" }, "Provider")
                select(attrs: attributes! { id=(&provider_id) name="provider" class="dns-provider-provider-select" disabled=(busy) },
                    <option value="cloudflare" selected=(!is_cloudns)>"Cloudflare"</option>
                    <option value="cloudns" selected=(is_cloudns)>"ClouDNS"</option>
                )
            </div>
            <div class="dns-provider-field grid min-w-0 items-center gap-2 sm:grid-cols-[12rem_minmax(0,26rem)]">
                label(attrs: attributes! { for=(&zone_id) class="dns-provider-required" }, "DNS zone")
                input(attrs: attributes! { id=(&zone_id) name="zone" value=(&values.zone) required=(true) aria-invalid=(values.zone.trim().is_empty()) disabled=(busy) })
            </div>
            <div class="dns-provider-cloudns grid gap-4 ">
                <div class="dns-provider-field grid min-w-0 items-center gap-2 sm:grid-cols-[12rem_minmax(0,26rem)]">
                    label(attrs: attributes! { for=(&auth_type_id) class="dns-provider-required" }, "Account type")
                    select(attrs: attributes! { id=(&auth_type_id) name="cloudns_auth_type" class="dns-provider-account-type-select" disabled=(busy) },
                        <option value="auth_id" selected=(!is_sub_auth)>"Account ID"</option>
                        <option value="sub_auth_id" selected=(is_sub_auth)>"Sub-account ID"</option>
                    )
                </div>
                <div class="dns-provider-field grid min-w-0 items-center gap-2 sm:grid-cols-[12rem_minmax(0,26rem)]">
                    label(attrs: attributes! { for=(&auth_id) class="dns-provider-required dns-provider-auth-id-label" }, (if is_sub_auth { "Sub-account ID" } else { "Account ID" }))
                    input(attrs: attributes! { id=(&auth_id) name="cloudns_auth_id" value=(cloudns_auth_id) required=(is_cloudns) aria-invalid=(is_cloudns && cloudns_auth_id.trim().is_empty()) disabled=(busy) })
                </div>
            </div>
            <div class="dns-provider-field grid min-w-0 items-center gap-2 sm:grid-cols-[12rem_minmax(0,26rem)]">
                label(attrs: attributes! { for=(&credential_id) class="dns-provider-cloudflare-label dns-provider-credential-label" }, "API token")
                label(attrs: attributes! { for=(&credential_id) class="dns-provider-cloudns-label dns-provider-credential-label" }, "API password")
                input(attrs: attributes! { id=(&credential_id) name="credential" type="password" autocomplete="new-password" placeholder=(if values.credential_saved { "Saved — leave blank to keep" } else { "" }) required=(!values.credential_saved) aria-invalid=(!values.credential_saved) disabled=(busy) })
            </div>
            <div class="dns-provider-field dns-provider-ttl-field grid min-w-0 items-center gap-2 sm:grid-cols-[12rem_minmax(0,26rem)]">
                label(attrs: attributes! { for=(&ttl_id) class="dns-provider-required" }, "TTL")
                <div class="dns-provider-ttl-controls grid min-w-0 gap-2">
                    <div class="dns-provider-ttl-mode">
                        select(attrs: attributes! { aria-label="TTL mode" disabled=(busy) },
                            <option value="auto" selected=(ttl_auto)>"Auto"</option>
                            <option value="seconds" selected=(!ttl_auto)>"Custom"</option>
                        )
                    </div>
                    <div class="dns-provider-ttl flex min-w-0" hidden=(ttl_auto)>
                        input(attrs: attributes! { id=(&ttl_id) name="ttl" type="number" min="2" value=(&ttl) required=(true) readonly=(ttl_auto) disabled=(busy) aria-describedby=(format!("{ttl_id}-unit {ttl_id}-error {ttl_id}-enterprise")) })
                        <span class="dns-provider-ttl-steps">
                            <button type="button" class="dns-provider-ttl-step" data-ttl-step="up" aria-label="Increase TTL" disabled=(busy)>icon(data: iconify_icon!("lucide:chevron-up"), attrs: attributes! { class="size-3" })</button>
                            <button type="button" class="dns-provider-ttl-step" data-ttl-step="down" aria-label="Decrease TTL" disabled=(busy)>icon(data: iconify_icon!("lucide:chevron-down"), attrs: attributes! { class="size-3" })</button>
                        </span>
                        <span id=(format!("{ttl_id}-unit")) class="dns-provider-ttl-unit">"s"</span>
                    </div>
                    <p id=(format!("{ttl_id}-error")) class="dns-provider-ttl-error" role="status" hidden=(true)></p>
                    <p id=(format!("{ttl_id}-enterprise")) class="dns-provider-ttl-enterprise" role="status" hidden=(true)>"TTLs below 60 seconds require a Cloudflare Enterprise subscription."</p>
                </div>
            </div>
            <div class="dns-provider-actions flex items-center gap-2">
                button(variant: ButtonVariant::Secondary, attrs: attributes! { type="submit" disabled=(true) },
                    icon(data: button_icon, attrs: attributes! { class=(icon_class) }) (button_text)
                )
            </div>
            </fieldset>
        </form>
    })
}
