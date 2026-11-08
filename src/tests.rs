use crate::{ClouDnsAuth, ClouDnsAuthId, ClouDnsPassword, ClouDnsSubAuthId, DnsProvider};

#[test]
fn cloudns_requires_an_id_and_password() {
    for input in ["", "   "] {
        assert!(input.parse::<ClouDnsAuthId>().is_err());
        assert!(input.parse::<ClouDnsSubAuthId>().is_err());
    }
    for input in ["", "   "] {
        assert!(input.parse::<ClouDnsPassword>().is_err());
    }
    let password: ClouDnsPassword = " secret ".parse().unwrap();
    assert_eq!(password.as_str(), " secret ");
    assert!(!format!("{password:?}").contains("secret"));
    assert!(
        DnsProvider::cloudns(
            ClouDnsAuth::AuthId(&"42".parse().unwrap()),
            &password,
            "example.com".parse().unwrap(),
            None
        )
        .is_ok()
    );
    assert!(
        DnsProvider::cloudns(
            ClouDnsAuth::SubAuthId(&"42".parse().unwrap()),
            &password,
            "example.com".parse().unwrap(),
            None
        )
        .is_ok()
    );
}

#[test]
fn names_must_stay_in_zone() {
    let provider = DnsProvider::cloudns(
        ClouDnsAuth::AuthId(&"42".parse().unwrap()),
        &"secret".parse().unwrap(),
        "Example.COM.".parse().unwrap(),
        None,
    )
    .unwrap();
    assert!(provider.check_name("_acme-challenge.example.com.").is_ok());
    assert!(provider.check_name("notexample.com").is_err());
    assert!(provider.check_name("example.com.evil.test").is_err());
}

#[test]
fn zone_names_are_normalized_and_checked() {
    use crate::ZoneName;
    assert_eq!(
        " Example.COM. ".parse::<ZoneName>().unwrap().as_str(),
        "example.com"
    );
    for invalid in [
        "",
        ".",
        "example..com",
        "-example.com",
        "example.com..",
        "éxample.com",
        "bad_name.example",
    ] {
        assert!(invalid.parse::<ZoneName>().is_err(), "{invalid}");
    }
    assert!(
        format!("{}.com", "a".repeat(64))
            .parse::<ZoneName>()
            .is_err()
    );
}

#[test]
fn ttl_parsing_and_provider_rules() {
    use crate::Ttl;
    assert_eq!("1".parse::<Ttl>().unwrap(), Ttl::AUTO);
    for input in ["", "0", "-1", "abc", "4294967296"] {
        assert!(input.parse::<Ttl>().is_err());
    }
    assert!(Ttl::seconds(1).is_err());
    assert!(Ttl::AUTO.validate_cloudflare().is_ok());
    assert!(Ttl::AUTO.validate_cloudns().is_err());
    assert!(Ttl::seconds(30).unwrap().validate_cloudflare().is_ok());
    assert!(Ttl::seconds(29).unwrap().validate_cloudflare().is_err());
    assert!(Ttl::seconds(86401).unwrap().validate_cloudflare().is_err());
    assert!(Ttl::seconds(300).unwrap().validate_cloudns().is_ok());
    assert!(Ttl::seconds(301).unwrap().validate_cloudns().is_err());
}

#[test]
fn cloudflare_token_is_validated_and_redacted() {
    use crate::{CloudflareToken, ZoneName};
    assert!("".parse::<CloudflareToken>().is_err());
    assert!("   ".parse::<CloudflareToken>().is_err());
    let token: CloudflareToken = "secret-token".parse().unwrap();
    assert!(!format!("{token:?}").contains("secret-token"));
    let zone: ZoneName = "Example.COM.".parse().unwrap();
    assert!(DnsProvider::cloudflare(&token, zone, None).is_ok());
}

#[test]
fn cloudns_password_serialization_is_compatible_and_validated() {
    let password: ClouDnsPassword = " secret ".parse().unwrap();
    let json = serde_json::to_string(&password).unwrap();
    assert_eq!(json, "\" secret \"");
    let restored: ClouDnsPassword = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, password);
    assert!(serde_json::from_str::<ClouDnsPassword>("\"   \"").is_err());
}
