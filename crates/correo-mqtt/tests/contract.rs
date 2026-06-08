use std::convert::TryFrom;

use correo_mqtt::{
    ConnectionId, MqttAuth, MqttConnectionOptions, MqttEndpoint, MqttError, MqttErrorKind,
    MqttProtocolVersion, PublishRequest, Qos, SecretBytes, SecretString, SshAuth, SshHostKeyPolicy,
    SshTunnelOptions, Subscription, TlsClientIdentity, TlsConfig, TlsHostVerification, TlsOptions,
    TlsTrustRoots,
};

#[test]
fn error_display_debug_and_reports_redact_sensitive_material() {
    let password = "synthetic-password-value";
    let token = "synthetic-token-value";
    let key = "synthetic-private-key-value";
    let error = MqttError::auth(format!(
        "broker rejected password={password}; token: {token}\n\
         -----BEGIN PRIVATE KEY-----\n{key}\n-----END PRIVATE KEY-----"
    ));

    assert_eq!(error.kind(), MqttErrorKind::Authentication);

    let display = error.to_string();
    let debug = format!("{error:?}");
    let report = error.to_report();
    let diagnostic = error.diagnostic_message();

    for rendered in [display, debug, report.message, diagnostic] {
        assert!(!rendered.contains(password), "{rendered}");
        assert!(!rendered.contains(token), "{rendered}");
        assert!(!rendered.contains(key), "{rendered}");
        assert!(!rendered.contains("PRIVATE KEY"), "{rendered}");
        assert!(rendered.contains("[REDACTED]"), "{rendered}");
    }
}

#[test]
fn connection_options_debug_redacts_runtime_secret_wrappers() {
    let password = "synthetic-password-value";
    let passphrase = "synthetic-passphrase-value";
    let private_key = b"synthetic-private-key-value";
    let mut options = MqttConnectionOptions::new(
        ConnectionId::new(),
        "local",
        MqttEndpoint::new("localhost", 1883).expect("valid endpoint"),
    );
    options.auth = MqttAuth::UsernamePassword {
        username: Some("synthetic-user".to_owned()),
        password: SecretString::new(password),
    };
    options.tls = TlsConfig::Enabled(TlsOptions {
        host_verification: TlsHostVerification::Enabled,
        trust_roots: TlsTrustRoots::Native,
        client_identity: Some(TlsClientIdentity::Pem {
            certificate_pem: SecretBytes::new(b"synthetic-cert".to_vec()),
            private_key_pem: SecretBytes::new(private_key.to_vec()),
        }),
    });
    options.ssh_tunnel = Some(SshTunnelOptions {
        host: "jump.example".to_owned(),
        port: 22,
        username: "ssh-user".to_owned(),
        auth: SshAuth::PrivateKey {
            path: Some("id_ed25519".to_owned()),
            private_key: Some(SecretBytes::new(private_key.to_vec())),
            passphrase: Some(SecretString::new(passphrase)),
        },
        host_key_policy: SshHostKeyPolicy::AcceptAnyInsecure,
        local_bind_port: None,
    });

    let debug = format!("{options:?}");

    assert!(!debug.contains(password), "{debug}");
    assert!(!debug.contains(passphrase), "{debug}");
    assert!(!debug.contains("synthetic-private-key-value"), "{debug}");
    assert!(debug.contains("SecretString(<redacted>)"), "{debug}");
    assert!(debug.contains("SecretBytes(<redacted>)"), "{debug}");
}

#[test]
fn protocol_and_qos_conversions_are_explicit() {
    assert_eq!(
        MqttProtocolVersion::try_from("3.1.1").expect("valid protocol"),
        MqttProtocolVersion::Mqtt3_1_1
    );
    assert_eq!(
        MqttProtocolVersion::try_from("MQTT 5").expect("valid protocol"),
        MqttProtocolVersion::Mqtt5
    );
    assert_eq!(
        MqttProtocolVersion::try_from("MQTT v5").expect("valid protocol"),
        MqttProtocolVersion::Mqtt5
    );
    assert_eq!(MqttProtocolVersion::Mqtt3_1_1.wire_level(), 4);
    assert_eq!(MqttProtocolVersion::Mqtt5.wire_level(), 5);

    assert_eq!(Qos::try_from(0).expect("valid qos"), Qos::AtMostOnce);
    assert_eq!(Qos::try_from(1).expect("valid qos"), Qos::AtLeastOnce);
    assert_eq!(Qos::try_from(2).expect("valid qos"), Qos::ExactlyOnce);
    assert_eq!(u8::from(Qos::ExactlyOnce), 2);
    assert_eq!(
        Qos::try_from(3).expect_err("invalid qos").kind(),
        MqttErrorKind::Protocol
    );
}

#[test]
fn request_builders_validate_publish_topics_and_subscription_filters() {
    let publish = PublishRequest::new(
        "devices/alpha/state",
        b"online".to_vec(),
        Qos::AtLeastOnce,
        true,
    )
    .expect("valid publish request");
    assert_eq!(publish.topic.as_str(), "devices/alpha/state");
    assert!(PublishRequest::new("devices/+/state", Vec::new(), Qos::AtMostOnce, false).is_err());

    let subscription =
        Subscription::new("devices/+/state", Qos::AtLeastOnce).expect("valid subscription filter");
    assert_eq!(subscription.topic_filter.as_str(), "devices/+/state");
    assert!(Subscription::new("devices/#/state", Qos::AtMostOnce).is_err());
}
