use serde_sarif::sarif::{Result as SarifResult, Sarif};
use std::collections::HashMap;

type FingerprintKey = (String, String);

// Performance has been prioritised over using a more functional (and readable) style.
// Comments explain choices made for performance improvements.
pub fn merge(new_sarif: &Sarif, old_sarif: &Sarif) -> Sarif {
    // Precompute lookup table to avoid repeated scans of old_sarif
    let fp_map = build_fingerprint_map(old_sarif);

    // Preallocate to avoid Vec reallocation during push
    let mut merged_runs = Vec::with_capacity(new_sarif.runs.len());

    for run in &new_sarif.runs {
        merged_runs.push(merge_run(run, &fp_map));
    }

    Sarif {
        runs: merged_runs,
        ..new_sarif.clone()
    }
}

#[inline]
fn build_fingerprint_map<'a>(sarif: &'a Sarif) -> HashMap<FingerprintKey, &'a SarifResult> {
    let mut fp_map = HashMap::new();

    // Imperative loops used to avoid iterator chaining overhead
    for run in &sarif.runs {
        let Some(results) = &run.results else {
            continue;
        };

        for result in results {
            let Some(fps) = &result.fingerprints else {
                continue;
            };

            // Clone keys eagerly to allow owned HashMap keys
            for (k, v) in fps {
                fp_map.insert((k.clone(), v.clone()), result);
            }
        }
    }

    fp_map
}

#[inline]
fn merge_run(
    run: &serde_sarif::sarif::Run,
    fp_map: &HashMap<FingerprintKey, &SarifResult>,
) -> serde_sarif::sarif::Run {
    let mut new_run = run.clone();

    let Some(results) = &run.results else {
        return new_run;
    };

    // Preallocate exact size to avoid growth checks
    let mut new_results = Vec::with_capacity(results.len());

    for result in results {
        new_results.push(merge_result(result, fp_map));
    }

    new_run.results = Some(new_results);
    new_run
}

#[inline]
fn merge_result(
    result: &SarifResult,
    fp_map: &HashMap<FingerprintKey, &SarifResult>,
) -> SarifResult {
    let mut updated = result.clone();

    let Some(fps) = &result.fingerprints else {
        return updated;
    };

    // Early-exit loop avoids unnecessary comparisons once a match is found
    for (k, v) in fps {
        if let Some(old) = fp_map.get(&(k.clone(), v.clone())) {
            updated.message = old.message.clone();
            updated.rank = old.rank.clone();
            updated.suppressions = old.suppressions.clone();
            break;
        }
    }

    updated
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
//keeps new sarif if no fingerprint matched

#[test]
fn merge_copies_rank_from_matching_fingerprint() {
    let new_sarif: Sarif = serde_json::from_str(
        r#"{
            "version": "2.1.0",
            "runs": [{
                "tool": { "driver": { "name": "new_sarif" } },
                "results": [{
                    "message": { "text": "new sarif message" },
                    "fingerprints": {
                        "hashResult/v1": "abc123"
                    }
                }]
            }]
        }"#,
    )
    .unwrap();

    let old_sarif: Sarif = serde_json::from_str(
        r#"{
            "version": "2.1.0",
            "runs": [{
                "tool": { "driver": { "name": "old_sarif" } },
                "results": [{
                    "message": { "text": "old sarif message" },
                    "rank": 5,
                    "fingerprints": {
                        "hashResult/v1": "abc123"
                    }
                }]
            }]
        }"#,
    )
    .unwrap();

    let merged = merge(&new_sarif, &old_sarif);

    assert_eq!(Some(5.0), merged.runs[0].results.as_ref().unwrap()[0].rank);
}

#[test]
fn merge_copies_suppressions_from_matching_fingerprint() {
    let new_sarif: Sarif = serde_json::from_str(
        r#"{
            "version": "2.1.0",
            "runs": [{
                "tool": { "driver": { "name": "new_sarif" } },
                "results": [{
                    "message": { "text": "new sarif message" },
                    "fingerprints": {
                        "hashResult/v1": "abc123"
                    }
                }]
            }]
        }"#,
    )
    .unwrap();

    let old_sarif: Sarif = serde_json::from_str(
        r#"{
            "version": "2.1.0",
            "runs": [{
                "tool": { "driver": { "name": "old_sarif" } },
                "results": [{
                    "message": { "text": "old sarif message" },
                    "fingerprints": {
                        "hashResult/v1": "abc123"
                    },
                    "suppressions": [{
                        "kind": "inSource",
                        "justification": "old justification"
                    }]
                }]
            }]
        }"#,
    )
    .unwrap();

    let merged = merge(&new_sarif, &old_sarif);

    let justification = merged.runs[0].results.as_ref().unwrap()[0]
        .suppressions
        .as_ref()
        .unwrap()[0]
        .justification
        .as_ref()
        .unwrap();

    assert_eq!("old justification", justification);
}

#[test]
#[ignore]
fn perf_merge_large_sarif() {
    let size: usize = std::env::var("SARIF_PERF_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10000);

    fn build_sarif(size: usize, prefix: usize, msg_prefix: &str) -> String {
        let mut s = String::with_capacity(size * 64);
        s.push_str("{\"version\":\"2.1.0\",\"runs\":[{");
        s.push_str("\"tool\":{\"driver\":{\"name\":\"perf\"}},\"results\":[");

        for i in 0..size {
            let fp = prefix + i;
            let res = format!(
                "{{\"message\":{{\"text\":\"{} {}\"}},\"fingerprints\":{{\"hashResult/v1\":\"{}\"}}}}",
                msg_prefix, i, fp
            );
            s.push_str(&res);
            if i + 1 < size {
                s.push(',');
            }
        }

        s.push_str("]}]}");
        s
    }

    // old has messages to copy from, new has placeholder messages
    let new_json = build_sarif(size, 0, "new");
    let old_json = build_sarif(size, 0, "old");

    let new_sarif: Sarif = serde_json::from_str(&new_json).expect("failed parsing new sarif");
    let old_sarif: Sarif = serde_json::from_str(&old_json).expect("failed parsing old sarif");

    let start = std::time::Instant::now();
    let _merged = merge(&new_sarif, &old_sarif);
    let dur = start.elapsed();

    eprintln!("perf_merge_large_sarif: size={} elapsed={:?}", size, dur);
}
