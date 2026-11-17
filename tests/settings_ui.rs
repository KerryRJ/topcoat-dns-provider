//! Focused tests for the API token status rendered by the provider settings form.

use std::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
};

use topcoat::{
    context::Cx,
    view::{ViewExt, view},
};
use topcoat_dns_provider::{
    ClouDnsAuthentication,
    settings_ui::{
        PROVIDER_FORM_CSS, PROVIDER_FORM_JS, ProviderFormProvider, ProviderFormValues,
        dns_provider_form,
    },
};

fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    for _ in 0..100 {
        if let Poll::Ready(output) = future.as_mut().poll(&mut cx) {
            return output;
        }
        std::thread::yield_now();
    }
    panic!("the provider form must render without external wakeups");
}

fn render(values: &ProviderFormValues) -> String {
    let cx = &Cx::new();
    let form = view! { dns_provider_form(id: "test", values: values) };
    block_on(form.single()).unwrap().render(cx).to_string()
}

fn cloudflare(credential_saved: bool) -> ProviderFormValues {
    ProviderFormValues {
        credential_saved,
        ..Default::default()
    }
}

fn cloudns(credential_saved: bool) -> ProviderFormValues {
    ProviderFormValues {
        provider: ProviderFormProvider::Cloudns {
            authentication: ClouDnsAuthentication::AuthId("12345".parse().unwrap()),
        },
        credential_saved,
        ..Default::default()
    }
}

/// Markup between the API token label and the API password label.
fn token_area(html: &str) -> &str {
    let start = html.find("API token").expect("API token label");
    let end = html.find("API password").expect("API password label");
    &html[start..end]
}

fn credential_input(html: &str) -> &str {
    let name = html.find(r#"name="credential""#).expect("credential input");
    let start = html[..name].rfind('<').expect("input start");
    let end = name + html[name..].find('>').expect("input end");
    &html[start..=end]
}

fn has_attr(element: &str, attr: &str) -> bool {
    element
        .split_whitespace()
        .any(|part| part == attr || part.starts_with(&format!("{attr}=")))
}

#[test]
fn cloudflare_token_is_plain_status_text() {
    let missing = render(&cloudflare(false));
    let area = token_area(&missing);
    assert!(area.contains(r#"data-credential-status="missing""#));
    assert!(area.contains(">Not stored in vault</p>"));
    assert!(area.contains("dns-provider-cloudflare-label"));
    assert!(area.contains(r#"role="status""#));
    assert!(
        area.starts_with("API token</span>"),
        "the API token caption must be plain span text, not a label: {area}"
    );
    assert!(
        !area.contains("dns-provider-required"),
        "the vault status must not carry a required marker: {area}"
    );
    assert!(
        !area.contains("<input"),
        "the API token must not render a textbox: {area}"
    );
    assert_eq!(
        missing.matches(r#"type="password""#).count(),
        1,
        "only the ClouDNS password field may be a textbox"
    );

    let saved = render(&cloudflare(true));
    let area = token_area(&saved);
    assert!(area.contains(r#"data-credential-status="stored""#));
    assert!(area.contains(">Stored in vault</p>"));
    assert!(!area.contains("<input"));
}

#[test]
fn cloudns_password_field_is_preserved() {
    let missing = render(&cloudns(false));
    let input = credential_input(&missing);
    assert!(input.contains(r#"type="password""#));
    assert!(has_attr(input, "required"));
    assert!(input.contains(r#"placeholder="""#));

    let saved = render(&cloudns(true));
    let input = credential_input(&saved);
    assert!(input.contains(r#"type="password""#));
    assert!(!has_attr(input, "required"));
    assert!(input.contains("Saved — leave blank to keep"));
    assert!(!has_attr(input, "aria-invalid"));
}

#[test]
fn saved_cloudns_password_does_not_mark_cloudflare_token_as_stored() {
    let html = render(&cloudns(true));
    let area = token_area(&html);
    assert!(area.contains(r#"data-credential-status="missing""#));
    assert!(area.contains(">Not stored in vault</p>"));
    assert!(!area.contains(r#"data-credential-status="stored""#));
}

#[test]
fn credential_input_is_disabled_outside_cloudns() {
    for values in [cloudflare(false), cloudflare(true)] {
        let html = render(&values);
        let input = credential_input(&html);
        assert!(
            has_attr(input, "disabled"),
            "Cloudflare must not submit the hidden ClouDNS password: {input}"
        );
    }

    let html = render(&cloudns(false));
    let input = credential_input(&html);
    assert!(!has_attr(input, "disabled"));
}

#[test]
fn required_markers_target_labels_only() {
    assert!(PROVIDER_FORM_CSS.contains("label.dns-provider-required::after"));
    assert!(PROVIDER_FORM_CSS.contains("label.dns-provider-cloudns-label::after"));
    assert!(!PROVIDER_FORM_CSS.contains("dns-provider-cloudflare-label::after"));
}

#[test]
fn cloudflare_short_ttl_uses_the_saved_vault_token() {
    assert!(
        PROVIDER_FORM_JS.contains("credential: ''"),
        "the plan request must send a blank credential so the host reads the vault"
    );
    assert!(
        PROVIDER_FORM_JS.contains("credential.required = cloudns && !cloudnsSaved;"),
        "the credential input must only be required for ClouDNS"
    );
    assert!(
        PROVIDER_FORM_JS.contains(
            "credential.disabled = !cloudns || form.getAttribute('aria-busy') === 'true';"
        ),
        "the hidden ClouDNS password must be disabled while Cloudflare is selected"
    );
    assert!(
        !PROVIDER_FORM_JS.contains("credential.required = provider !== form.dataset.savedProvider"),
        "Cloudflare must not require a token input"
    );
    assert!(
        PROVIDER_FORM_JS.contains("form.dataset.savedProvider === 'cloudflare'"),
        "vault status must be scoped to the saved provider"
    );
    assert!(
        PROVIDER_FORM_JS.contains(": form.dataset.savedProvider === 'cloudflare';"),
        "Cloudflare Save must require a token in the saved vault"
    );
}
