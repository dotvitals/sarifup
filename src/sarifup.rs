use serde_sarif::sarif::{Result as SarifResult, Run, Sarif};
use std::collections::HashMap;

pub fn merge(new_sarif: &Sarif, old_sarif: &Sarif) -> Sarif {
    // Build fingerprint → old result map
    let old_by_fingerprint: HashMap<(String, String), SarifResult> = old_sarif
        .runs
        .iter()
        .flat_map(|run| run.results.iter().flatten())
        .flat_map(|result| {
            result
                .fingerprints
                .iter()
                .flatten()
                .map(move |(k, v)| ((k.clone(), v.clone()), result.clone()))
        })
        .collect();

    // Helper: replace message when fingerprint matches
    let update_result = |mut result: SarifResult| {
        if let Some(fps) = &result.fingerprints {
            for (k, v) in fps {
                if let Some(old) = old_by_fingerprint.get(&(k.clone(), v.clone())) {
                    result.message = old.message.clone();
                    break;
                }
            }
        }
        result
    };

    // Build merged SARIF
    let merged_runs: Vec<Run> = new_sarif
        .runs
        .iter()
        .cloned()
        .map(|mut run| {
            run.results = run
                .results
                .map(|results| results.into_iter().map(update_result).collect());
            run
        })
        .collect();

    Sarif {
        runs: merged_runs,
        ..new_sarif.clone()
    }
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
