# topcoat-dns-provider

A reusable Rust DNS provider client with asynchronous record operations. Applications are responsible for authorization and secure storage of provider credentials.

```toml
[dependencies]
topcoat-dns-provider = { path = "../topcoat-dns-provider" }
```

```rust,no_run
use topcoat_dns_provider::{CloudflareToken, DnsProvider, RecordData, RecordType, Ttl, ZoneName};

# async fn example() {
let token: CloudflareToken = "api-token".parse().unwrap();
let zone: ZoneName = "example.com".parse().unwrap();
let dns = DnsProvider::cloudflare(&token, zone, None).unwrap();
let record = RecordData::TXT("example value".into());
dns.add("example.com", Ttl::seconds(300).unwrap(), record.clone()).await.unwrap();
let records = dns.list("example.com", RecordType::TXT).await.unwrap();
dns.remove("example.com", record).await.unwrap();
# }
```

Use `DnsProvider::cloudns` with `ClouDnsAuth::AuthId(&auth_id)` or `ClouDnsAuth::SubAuthId(&sub_auth_id)` and a `ClouDnsPassword` for ClouDNS. Parse the API password into this type before use. Parse IDs into `ClouDnsAuthId` or `ClouDnsSubAuthId` respectively; both reject empty or whitespace-only values. Stored authentication uses the same types and retains its existing string representation. Constructors do not contact the provider. Names passed to operations must belong to the configured zone. Removal targets the supplied value rather than the whole RRset. Provider permissions, TTL rules, and supported types vary.

Retain a `DnsProvider` when performing repeated operations so its HTTP client can reuse connections. For multiple values of the same name and type, `add_many` and `remove_many` share the provider's existing-record lookup. They reject mixed record types and ignore duplicate input values. These operations are not atomic: a provider error may occur after some values have been changed, and concurrent updates may affect the result. Removal preserves values not requested for removal.

The underlying `dns-update` library currently models A, AAAA, CNAME, NS, MX, TXT, SRV, TLSA and CAA. The requested complete set of official record types has not yet been implemented. The reusable `settings_ui::dns_provider_form` renders provider settings separately from an application domain. Topcoat button, input, label, and select components are installed and exported through `components`. The optional `fonts::GEIST` Fontsource font can be registered and linked by a host Topcoat application; hosts control their own theme and assets.

The structured record data types are available as `Mx`, `Srv`, `Tlsa`, and `Caa`.

Run `cargo test` to test the crate. Successful local tests do not verify live provider access.



## Settings storage

Implement `DnsProviderStore` for an application-owned, scoped store. Its async
`load` and `save` methods exchange `DnsProviderSettings`, including the enabled state,
zone, TTL and typed Cloudflare or ClouDNS credentials. `load` returns `None` if
there is no saved configuration; `save` replaces the full configuration. Preserve
a saved credential before calling `save` when a form submits a blank secret.
Implementations must authorize access and encrypt credentials at rest. Never return
these settings directly to a browser or write them to logs. Stores select their own
application or tenant scope. The trait supports generic callers and Send futures;
it is not object-safe. The crate does not provide a database implementation.

Hosts can call `settings.enabled_provider(None)?` to obtain `Some(DnsProvider)`
when enabled or `None` when disabled, and use `settings.ttl` for writes. Reload
settings and replace any cached client after a change; disabling does not revoke
clients already constructed. Hosts using the direct constructors must check
`enabled` themselves before DNS operations.
Disabling retains the configuration and credentials. Existing serialized settings
without this field default to enabled for compatibility.

## Provider settings form

Use `settings_ui::dns_provider_form` with a unique `id`, a `ProviderFormValues` containing only non-secret values (use `ProviderFormProvider::Cloudflare` or `ProviderFormProvider::Cloudns { authentication }`, with `ClouDnsAuthentication` for the ClouDNS account ID), and form `attrs` specifying the host's action and method. Set `ProviderFormValues.enabled` from the saved `DnsProviderSettings.enabled`. The checkbox submits `enabled=true` when checked and omits it when unchecked; the host must treat an absent `enabled` field as false and persist it. The form also submits `provider`, `zone`, `ttl`, `cloudns_auth_type`, `cloudns_auth_id`, and `credential`. ClouDNS account fields are shown when ClouDNS is selected. The host validates the fields and handles the submission asynchronously. A blank credential can retain a saved credential only when the provider is unchanged; require a new credential when changing providers. Never populate the password field with a stored secret. Set `credential_saved` only for the currently saved provider.

Include `settings_ui::PROVIDER_FORM_CSS` and `settings_ui::PROVIDER_FORM_JS` once on pages containing the form. The script switches between Cloudflare's automatic TTL (submitted as `1`) and a user-entered TTL in seconds, and returns to seconds when switching to ClouDNS.

For Cloudflare TTLs from 30 to 59 seconds, provide `data-cloudflare-plan-url` in the form attributes with a same-origin, application-owned POST endpoint. The browser sends JSON `{ "zone": "example.com", "credential": "..." }`; return `{ "enterprise": true }` or `{ "enterprise": false }`, or a non-success status if the subscription cannot be checked. A blank credential means use the securely stored Cloudflare token only if it belongs to the current configuration. The endpoint must authenticate and authorize the caller, enforce the application's CSRF policy, avoid logging request bodies or tokens, and return no saved credentials. Call the async `check_cloudflare_enterprise(&token, &zone)` helper and return only its boolean result. If the check fails or is unavailable, the form changes a short TTL to 60 seconds and explains why. Repeat the check on the server when saving a short TTL; browser state is not authorization or validation. For dynamically inserted forms, dispatch the refresh event described below.

### Cloudflare configuration

1. Add the domain as a Cloudflare zone. Use an Enterprise zone for DNS-only TTLs below 60 seconds.
2. Create an API token restricted to the intended zone and account. Grant **Zone: DNS: Edit** for record operations and **Zone: Zone: Read** for zone lookup. The subscription check also requires **Account: Billing: Read**; grant only the necessary account scope. Cloudflare's [zone subscription endpoint](https://developers.cloudflare.com/api/resources/zones/subresources/subscriptions/methods/get/) documents the Billing permission. If your account cannot grant it or Cloudflare does not return a recognized active subscription, the check remains unverified; use Auto or at least 60 seconds.
3. Enter the zone name and token in the application. Store the token encrypted in the application, never in browser storage or logs. The short-TTL check reads subscription metadata; it does not create or modify DNS records. A successful plan check does not guarantee a later DNS write will succeed.

The script enables Save only when the form has valid changes. After a successful save, re-render the form with the saved values so they become the new baseline; after inserting or changing a form dynamically, dispatch a bubbling `dns-provider:refresh` event on it. Hosts must still validate and authorize submissions server-side.

Changing the Enable switch automatically submits the form with `settings_action=enabled`, bypassing provider-field validation. The host must capture the submitted form data synchronously in its submit handler, persist only the enabled state, retain the existing provider configuration and credentials, and re-render with the persisted state after success or failure. An unchecked switch omits `enabled`; treat that as false. Disabled provider fields are omitted from submissions. Normal Save submissions use `settings_action=save` and require the usual validation. Disable the switch while either request is in flight. The provider fields and Save button are disabled whenever the switch is off.

Set `busy` while saving to disable the controls and display progress; the host owns the submission state and feedback. Supply a unique ID per form. The form uses Topcoat semantic theme tokens and `dns-provider-*` CSS classes, and forwards form attributes including additional classes. Include `settings_ui::PROVIDER_FORM_CSS` in the host stylesheet for provider-specific field visibility, and include this crate's source in the host's Tailwind class scanning so its utility classes are generated. The host owns theme selection and can override the classes. Application-domain settings are a separate host component and are not saved by this form.

After a successful application-domain save, the host should publish the persisted domain through its own signal or event mechanism so dependent components can refresh. This crate does not own application-domain state or dispatch that event.




`CloudflareToken` and `ClouDnsPassword` reject empty or whitespace-only credentials and redact debug output. Their serialized representations contain secrets and must be protected. Provider constructors accept a validated `ZoneName`; Cloudflare also requires a `CloudflareToken`, and ClouDNS requires a `ClouDnsPassword`. This changes their previous string-based signatures.

`ZoneName` validates ASCII DNS zone labels, normalizes case, and removes one trailing root dot. Convert international names to IDNA ASCII before parsing. `Ttl` distinguishes `Ttl::AUTO` (Cloudflare's value 1) from seconds; provider operations check documented provider limits. Cloudflare TTLs from 30 to 59 seconds require an Enterprise zone and may be rejected by the provider. ClouDNS values follow its documented list and may depend on account capabilities. Form zone and TTL values are strings so invalid input can be shown again; parse them into `ZoneName` and `Ttl`, then call `validate_cloudflare` or `validate_cloudns` before saving. The host must also validate credentials and ClouDNS IDs. Existing stored numeric TTL and string zone representations are retained, but invalid stored values will fail deserialization.

## Browser preview

Run `cargo run --manifest-path preview/Cargo.toml` and open <http://127.0.0.1:3000>. The first build may download the Tailwind executable. The preview renders the real component with the Topcoat neutral theme; toggle the theme, resize the browser, and switch providers to inspect the layout. Save simulates a short operation and does not store form data. For a Cloudflare TTL below 60, the local preview sends the entered zone and token to Cloudflare to check the subscription. Stop the server with Ctrl+C. The preview has no authentication and is intended for local use only.
