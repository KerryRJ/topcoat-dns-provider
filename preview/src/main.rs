use axum::{
    Json, Router,
    http::{StatusCode, header},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use topcoat::{
    context::Cx,
    icon::{icon, iconify::iconify_icon},
    view::{ViewExt, attributes, view},
};
use topcoat_dns_provider::components::button::{ButtonSize, ButtonVariant, button};
use topcoat_dns_provider::settings_ui::{
    PROVIDER_FORM_CSS, PROVIDER_FORM_JS, ProviderFormValues, dns_provider_form,
};

async fn index() -> Html<String> {
    let values = ProviderFormValues::default();
    let cx = &Cx::new();
    let form = view! { dns_provider_form(id: "preview", values: &values, attrs: attributes! { data-cloudflare-plan-url="/check-cloudflare-plan" }) }
        .single().await.unwrap().render(cx);
    let theme_button = view! {
        button(variant: ButtonVariant::Ghost, size: ButtonSize::Icon,
            attrs: attributes! { id="theme" type="button" aria-label="Switch to light theme" title="Switch to light theme" },
            icon(data: iconify_icon!("lucide:moon"), attrs: attributes! { class="theme-moon" aria-hidden="true" })
            icon(data: iconify_icon!("lucide:sun"), attrs: attributes! { class="theme-sun" aria-hidden="true" })
        )
    }.single().await.unwrap().render(cx);
    Html(format!(
        r#"<!doctype html><html lang="en" class="dark"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>DNS provider preview</title><link rel="stylesheet" href="/styles.css"><style>{PROVIDER_FORM_CSS} body {{ padding: 2rem; }} main {{ max-width: 50rem; margin: auto; }} header {{ display:flex; justify-content:space-between; align-items:center; gap:1rem; margin-bottom:2rem; }} h1 {{ font-size:1.5rem; font-weight:600; }} #status {{ margin-top:1rem; }} .theme-sun, .dark .theme-moon {{ display:none; }} .dark .theme-sun {{ display:block; }}</style></head><body class="bg-background text-foreground"><main><header><h1>DNS provider settings</h1>{theme_button}</header>{form}<p id="status" role="status"></p></main><script>
const form = document.querySelector('form');
{PROVIDER_FORM_JS}
document.querySelector('#theme').addEventListener('click', event => {{
 const dark = document.documentElement.classList.toggle('dark');
 const label = dark ? 'Switch to light theme' : 'Switch to dark theme';
 event.currentTarget.setAttribute('aria-label', label);
 event.currentTarget.title = label;
}});
form.addEventListener('submit', event => {{
 event.preventDefault();
 const controls = [...form.elements];
 const button = form.querySelector('button[type=submit]');
 const original = button.innerHTML;
 form.setAttribute('aria-busy', 'true');
 controls.forEach(control => control.disabled = true);
 button.innerHTML = '<span class="inline-block size-4 animate-spin" aria-hidden="true">⟳</span> Saving…';
 document.querySelector('#status').textContent = '';
 setTimeout(() => {{ controls.forEach(control => control.disabled = false); button.innerHTML = original; form.setAttribute('aria-busy', 'false'); form.querySelector('[name=credential]').value = ''; form.dispatchEvent(new Event('dns-provider:refresh', {{ bubbles: true }})); document.querySelector('#status').textContent = 'Preview saved. No data was stored or sent.'; }}, 1200);
}});
</script></body></html>"#
    ))
}

async fn styles() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!(concat!(env!("OUT_DIR"), "/tailwind.css")),
    )
}

#[derive(serde::Deserialize)]
struct PlanRequest {
    zone: String,
    credential: String,
}
#[derive(serde::Serialize)]
struct PlanResponse {
    enterprise: bool,
}
async fn check_plan(Json(request): Json<PlanRequest>) -> impl IntoResponse {
    let result = async {
        let zone = request.zone.parse().map_err(|_| ())?;
        let token = request.credential.parse().map_err(|_| ())?;
        topcoat_dns_provider::check_cloudflare_enterprise(&token, &zone)
            .await
            .map_err(|_| ())
    }
    .await;
    match result {
        Ok(enterprise) => (
            [(header::CACHE_CONTROL, "no-store")],
            Json(PlanResponse { enterprise }),
        )
            .into_response(),
        Err(()) => (
            StatusCode::BAD_REQUEST,
            [(header::CACHE_CONTROL, "no-store")],
            "Could not verify the Cloudflare zone subscription.",
        )
            .into_response(),
    }
}

#[tokio::main]
async fn main() {
    let address = std::env::var("DNS_PREVIEW_BIND").unwrap_or_else(|_| "127.0.0.1:3000".into());
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .expect("bind preview address");
    println!("Preview: http://{address}");
    axum::serve(
        listener,
        Router::new()
            .route("/", get(index))
            .route("/styles.css", get(styles))
            .route("/check-cloudflare-plan", post(check_plan)),
    )
    .await
    .unwrap();
}
