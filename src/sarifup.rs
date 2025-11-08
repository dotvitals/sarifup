use serde_sarif::sarif::Sarif;

pub fn merge(new_sarif: &Sarif, _old_sarif: &Sarif) -> Sarif {
    return new_sarif.clone();
}

#[test]
fn merge_returns_new_sarif() {
    let new_sarif: Sarif = serde_json::from_str(r#"{ "version": "2.1.0", "runs": [] }"#).unwrap();
    let old_sarif: Sarif = serde_json::from_str(
        r#"{ "version": "2.1.0", "runs": [{"tool": {"driver": {"name": "test"}}}] }"#,
    )
    .unwrap();

    let merged_sarif = merge(&new_sarif, &old_sarif);

    assert_eq!(new_sarif, merged_sarif);
}
