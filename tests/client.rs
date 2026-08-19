use namecheap_cli::api::{split_domain, AddRecordResult, ApiError, NamecheapClient};
use namecheap_cli::dns::{DnsRecord, RecordType};
use std::fs;
use wiremock::matchers::{body_string_contains, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

const DOMAIN: &str = "readyms.xyz";

fn get_hosts_xml(domain: &str, hosts: &[(&str, &str, &str, &str)]) -> String {
    let host_elems: String = hosts
        .iter()
        .enumerate()
        .map(|(i, (name, rtype, address, ttl))| {
            format!(
                r#"<host HostId="{}" Name="{}" Type="{}" Address="{}" MXPref="10" TTL="{}" />"#,
                i + 1,
                name,
                rtype,
                address,
                ttl
            )
        })
        .collect();

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<ApiResponse Status="OK">
  <CommandResponse Type="namecheap.domains.dns.getHosts">
    <DomainDNSGetHostsResult Domain="{domain}" IsUsingOurDNS="true">
      {host_elems}
    </DomainDNSGetHostsResult>
  </CommandResponse>
</ApiResponse>"#
    )
}

fn set_hosts_ok_xml(domain: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<ApiResponse Status="OK">
  <CommandResponse Type="namecheap.domains.dns.setHosts">
    <DomainDNSSetHostsResult Domain="{domain}" IsSuccess="true" />
  </CommandResponse>
</ApiResponse>"#
    )
}

fn test_client(server: &MockServer, backup_dir: &std::path::Path) -> NamecheapClient {
    NamecheapClient::from_parts(server.uri(), "testuser", "testkey", "testuser", "127.0.0.1")
        .expect("client")
        .with_backup_dir(backup_dir)
}

async fn mock_get_hosts(server: &MockServer, xml: String) {
    Mock::given(method("POST"))
        .and(body_string_contains(
            "Command=namecheap.domains.dns.getHosts",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_string(xml))
        .mount(server)
        .await;
}

async fn mock_set_hosts(server: &MockServer, xml: String) {
    Mock::given(method("POST"))
        .and(body_string_contains(
            "Command=namecheap.domains.dns.setHosts",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_string(xml))
        .mount(server)
        .await;
}

fn a_record(host: &str, value: &str) -> DnsRecord {
    DnsRecord::new(RecordType::A, host, value, 1800, None)
}

#[test]
fn test_split_domain_readyms_xyz() {
    let (sld, tld) = split_domain("readyms.xyz");
    assert_eq!(sld, "readyms");
    assert_eq!(tld, "xyz");
}

#[tokio::test]
async fn add_identical_type_host_value_is_noop() {
    let server = MockServer::start().await;
    let backup = tempfile::tempdir().unwrap();
    let client = test_client(&server, backup.path());

    mock_get_hosts(
        &server,
        get_hosts_xml(DOMAIN, &[("@", "A", "1.2.3.4", "1800")]),
    )
    .await;

    Mock::given(method("POST"))
        .and(body_string_contains(
            "Command=namecheap.domains.dns.setHosts",
        ))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;

    let result = client
        .add_record(DOMAIN, a_record("@", "1.2.3.4"), false)
        .await
        .expect("add");
    assert_eq!(result, AddRecordResult::Unchanged);
    assert!(fs::read_dir(backup.path()).unwrap().next().is_none());
}

#[tokio::test]
async fn add_different_value_same_type_host_errors() {
    let server = MockServer::start().await;
    let backup = tempfile::tempdir().unwrap();
    let client = test_client(&server, backup.path());

    mock_get_hosts(
        &server,
        get_hosts_xml(DOMAIN, &[("@", "A", "1.2.3.4", "1800")]),
    )
    .await;

    Mock::given(method("POST"))
        .and(body_string_contains(
            "Command=namecheap.domains.dns.setHosts",
        ))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;

    let err = client
        .add_record(DOMAIN, a_record("@", "5.6.7.8"), false)
        .await
        .expect_err("conflict");
    match err {
        ApiError::RecordConflict(message) => {
            assert!(message.contains("dns set"), "{message}");
            assert!(message.contains("1.2.3.4"), "{message}");
        }
        other => panic!("expected RecordConflict, got {other:?}"),
    }
}

#[tokio::test]
async fn add_new_record_calls_set_hosts() {
    let server = MockServer::start().await;
    let backup = tempfile::tempdir().unwrap();
    let client = test_client(&server, backup.path());

    mock_get_hosts(
        &server,
        get_hosts_xml(DOMAIN, &[("@", "A", "1.2.3.4", "1800")]),
    )
    .await;
    mock_set_hosts(&server, set_hosts_ok_xml(DOMAIN)).await;

    let result = client
        .add_record(DOMAIN, a_record("www", "9.9.9.9"), false)
        .await
        .expect("add");
    assert_eq!(result, AddRecordResult::Added);

    let set_requests: Vec<_> = server
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .filter(|r| {
            String::from_utf8_lossy(&r.body).contains("Command=namecheap.domains.dns.setHosts")
        })
        .collect();
    assert_eq!(set_requests.len(), 1);
    let body = String::from_utf8_lossy(&set_requests[0].body);
    assert!(body.contains("SLD=readyms"), "{body}");
    assert!(body.contains("TLD=xyz"), "{body}");
    assert!(
        body.contains("Address1=1.2.3.4") || body.contains("Address2=1.2.3.4"),
        "{body}"
    );
    assert!(
        body.contains("Address1=9.9.9.9") || body.contains("Address2=9.9.9.9"),
        "{body}"
    );
    assert!(backup.path().read_dir().unwrap().next().is_some());
}

#[tokio::test]
async fn set_record_replaces_matching_type_host() {
    let server = MockServer::start().await;
    let backup = tempfile::tempdir().unwrap();
    let client = test_client(&server, backup.path());

    mock_get_hosts(
        &server,
        get_hosts_xml(
            DOMAIN,
            &[
                ("@", "A", "1.2.3.4", "1800"),
                ("www", "TXT", "hello", "1800"),
            ],
        ),
    )
    .await;
    mock_set_hosts(&server, set_hosts_ok_xml(DOMAIN)).await;

    client
        .set_record(DOMAIN, a_record("@", "5.6.7.8"), false)
        .await
        .expect("set");

    let set_requests: Vec<_> = server
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .filter(|r| {
            String::from_utf8_lossy(&r.body).contains("Command=namecheap.domains.dns.setHosts")
        })
        .collect();
    assert_eq!(set_requests.len(), 1);
    let body = String::from_utf8_lossy(&set_requests[0].body);
    assert!(
        body.contains("Address1=5.6.7.8") || body.contains("Address2=5.6.7.8"),
        "{body}"
    );
    assert!(
        !body.contains("Address1=1.2.3.4") && !body.contains("Address2=1.2.3.4"),
        "{body}"
    );
    assert!(
        body.contains("Address1=hello") || body.contains("Address2=hello"),
        "{body}"
    );
    assert!(body.contains("SLD=readyms"), "{body}");
    assert!(body.contains("TLD=xyz"), "{body}");

    let backups: Vec<_> = fs::read_dir(backup.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(backups.len(), 1);
    let dumped: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&backups[0]).unwrap()).unwrap();
    assert_eq!(dumped["domain"], DOMAIN);
    assert_eq!(dumped["records"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn set_hosts_refuses_empty_zone_without_flag() {
    let server = MockServer::start().await;
    let backup = tempfile::tempdir().unwrap();
    let client = test_client(&server, backup.path());

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;

    let err = client
        .set_hosts(DOMAIN, &[], false)
        .await
        .expect_err("empty zone");
    match err {
        ApiError::EmptyZone(message) => {
            assert!(message.contains("--allow-empty"), "{message}");
            assert!(message.contains(DOMAIN), "{message}");
        }
        other => panic!("expected EmptyZone, got {other:?}"),
    }
    assert!(fs::read_dir(backup.path()).unwrap().next().is_none());
}

#[tokio::test]
async fn set_hosts_allow_empty_sends_set_hosts_without_records() {
    let server = MockServer::start().await;
    let backup = tempfile::tempdir().unwrap();
    let client = test_client(&server, backup.path());

    mock_get_hosts(
        &server,
        get_hosts_xml(DOMAIN, &[("@", "A", "1.2.3.4", "1800")]),
    )
    .await;
    mock_set_hosts(&server, set_hosts_ok_xml(DOMAIN)).await;

    client
        .set_hosts(DOMAIN, &[], true)
        .await
        .expect("allow empty");

    let set_requests: Vec<_> = server
        .received_requests()
        .await
        .unwrap()
        .into_iter()
        .filter(|r| {
            String::from_utf8_lossy(&r.body).contains("Command=namecheap.domains.dns.setHosts")
        })
        .collect();
    assert_eq!(set_requests.len(), 1);
    let body = String::from_utf8_lossy(&set_requests[0].body);
    assert!(body.contains("SLD=readyms"), "{body}");
    assert!(body.contains("TLD=xyz"), "{body}");
    assert!(!body.contains("HostName1"), "{body}");
    assert!(backup.path().read_dir().unwrap().next().is_some());
}

#[tokio::test]
async fn user_agent_uses_crate_version() {
    let server = MockServer::start().await;
    let backup = tempfile::tempdir().unwrap();
    let client = test_client(&server, backup.path());

    mock_get_hosts(&server, get_hosts_xml(DOMAIN, &[])).await;

    client.get_hosts(DOMAIN).await.expect("get hosts");

    let requests = server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    let ua = requests[0]
        .headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(ua, concat!("namecheap-cli/", env!("CARGO_PKG_VERSION")));
}
