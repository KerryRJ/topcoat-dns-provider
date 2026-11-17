//! Focused tests for the credential vault status rendered by the provider settings form.

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

/// Markup for the credential vault status paragraph.
fn credential_status(html: &str) -> &str {
    let marker = html
        .find("dns-provider-credential-status")
        .expect("credential status");
    let start = html[..marker].rfind('<').expect("status start");
    let end = marker + html[marker..].find("</p>").expect("status end") + "</p>".len();
    &html[start..end]
}

#[test]
fn credential_is_plain_vault_status() {
    for values in [
        cloudflare(false),
        cloudflare(true),
        cloudns(false),
        cloudns(true),
    ] {
        let html = render(&values);
        let status = credential_status(&html);
        assert!(status.contains(r#"role="status""#), "{status}");
        assert!(status.contains("text-muted-foreground"), "{status}");
        assert!(!status.contains("<input"), "{status}");
        assert!(
            !status.contains("dns-provider-required"),
            "the vault status must not carry a required marker: {status}"
        );
        let state = if values.credential_saved {
            "stored"
        } else {
            "missing"
        };
        assert!(
            status.contains(&format!(r#"data-credential-status="{state}""#)),
            "{status}"
        );
        let text = if values.credential_saved {
            ">Stored in vault</p>"
        } else {
            ">Not stored in vault</p>"
        };
        assert!(status.contains(text), "{status}");
    }
}

#[test]
fn form_never_collects_a_credential() {
    for values in [
        cloudflare(false),
        cloudflare(true),
        cloudns(false),
        cloudns(true),
    ] {
        let html = render(&values);
        assert!(
            !html.contains(r#"name="credential""#),
            "the form must not submit a credential: {html}"
        );
        assert!(
            !html.contains(r#"type="password""#),
            "the form must not render a password textbox: {html}"
        );
        assert!(
            !html.contains("API password"),
            "the password label must be plain text: {html}"
        );
    }
}

#[test]
fn credential_labels_are_plain_text() {
    let html = render(&cloudflare(false));
    assert!(
        html.contains(r#">API token</span>"#),
        "the Cloudflare caption must be plain span text: {html}"
    );
    assert!(
        html.contains(
            r#"dns-provider-cloudns-label dns-provider-credential-label">Password</span>"#
        ),
        "the ClouDNS caption must be plain span text: {html}"
    );
}

#[test]
fn required_markers_target_labels_only() {
    assert!(PROVIDER_FORM_CSS.contains("label.dns-provider-required::after"));
    assert!(!PROVIDER_FORM_CSS.contains("dns-provider-cloudns-label::after"));
    assert!(!PROVIDER_FORM_CSS.contains("dns-provider-credential-label::after"));
}

#[test]
fn short_ttl_uses_the_saved_vault_credential() {
    assert!(
        PROVIDER_FORM_JS.contains("credential: ''"),
        "the plan request must send a blank credential so the host reads the vault"
    );
    assert!(
        !PROVIDER_FORM_JS.contains(r#"[name="credential"]"#),
        "the form must not read a credential input"
    );
    assert!(
        PROVIDER_FORM_JS
            .contains("const credentialValid = form.dataset.savedProvider === provider;"),
        "Save must require a vaulted credential for the selected provider"
    );
    assert!(
        !PROVIDER_FORM_JS.contains("credential.required"),
        "the form must not require a password input"
    );
}
