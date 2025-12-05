use serde_sarif::sarif::{Result as SarifResult, Run, Sarif};

pub fn merge(new_sarif: &Sarif, old_sarif: &Sarif) -> Sarif {
    let mut merged = new_sarif.clone();

    // Build lookup table: (fingerprint_key, fingerprint_value) → old_result
    let mut old_by_fingerprint = std::collections::HashMap::<(String, String), SarifResult>::new();

    for run in &old_sarif.runs {
        if let Some(results) = &run.results {
            for r in results {
                if let Some(fps) = &r.fingerprints {
                    for (k, v) in fps.iter() {
                        old_by_fingerprint.insert((k.clone(), v.clone()), r.clone());
                    }
                }
            }
        }
    }

    // Modify merged new_sarif in-place
    for run in &mut merged.runs {
        if let Some(results) = &mut run.results {
            for result in results.iter_mut() {
                if let Some(fps) = &result.fingerprints {
                    for (k, v) in fps.iter() {
                        if let Some(old_result) = old_by_fingerprint.get(&(k.clone(), v.clone())) {
                            // Replace message only
                            result.message = old_result.message.clone();
                            break; // Only replace once per result
                        }
                    }
                }
            }
        }
    }

    merged
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

#[test]
fn merge_updates_message_from_matching_fingerprint_in_any_result_in_any_run() {
    let new_sarif: Sarif = serde_json::from_str(
        r#"{
            "version": "2.1.0",
            "runs": [{
                    "tool": {
                        "driver": {
                            "name": "new_sarif_1"
                        }
                    },
                    "results": [{
                           "message": {
                                "text": "new sarif message 1"
                            },
                            "fingerprints": {
                                "hashResult/v1": "abc123"
                            }
                        }, {
                            "message": {
                                "text": "new sarif message 2"
                            }
                        }
                    ]
                },{
                    "tool": {
                        "driver": {
                            "name": "new_sarif_2"
                        }
                    }
                }
            ]
        }"#,
    )
    .unwrap();

    let old_sarif: Sarif = serde_json::from_str(
        r#"{
            "version": "2.1.0",
            "runs": [{
                    "tool": {
                        "driver": {
                            "name": "old_sarif_1"
                        }
                    }
                }, {
                    "tool": {
                        "driver": {
                            "name": "old_sarif_2"
                        }
                    },
                    "results": [{
                            "message": {
                                "text": "old sarif message 1"
                            }
                        }, {
                            "message": {
                                "text": "old sarif message 2"
                            },
                            "fingerprints": {
                                "hashResult/v1": "abc123"
                            }
                        }
                    ]
                }
            ]
        }"#,
    )
    .unwrap();

    let merged_sarif = merge(&new_sarif, &old_sarif);

    assert_eq!(
        "old sarif message 2",
        merged_sarif.runs[0].results.as_ref().unwrap()[0]
            .message
            .text
            .as_ref()
            .unwrap()
    );
}

//returns rank from any result in any run
//keeps new sairf if no fingerprint matched
