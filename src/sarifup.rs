use serde_sarif::sarif::{Result as SarifResult, Sarif};
use std::collections::HashMap;

type FingerprintKey = (String, String);

#[inline]
fn count_run_changes(
    run: &serde_sarif::sarif::Run,
    fp_map: &HashMap<FingerprintKey, &SarifResult>,
    old_result_count: usize,
) -> (usize, usize, usize) {
    let Some(results) = &run.results else {
        return (0, 0, old_result_count);
    };

    let mut updated = 0;

    for result in results {
        let Some(fps) = &result.fingerprints else {
            continue;
        };

        for (k, v) in fps {
            if fp_map.contains_key(&(k.clone(), v.clone())) {
                updated += 1;
                break;
            }
        }
    }

    let new = results.len() - updated;
    let closed = old_result_count.saturating_sub(updated);

    (new, updated, closed)
}

#[inline]
fn count_old_results(sarif: &Sarif) -> usize {
    let mut count = 0;

    for run in &sarif.runs {
        if let Some(results) = &run.results {
            count += results.len();
        }
    }

    count
}

// Performance has been prioritised over using a more functional (and readable) style.
// Comments explain choices made for performance improvements.
pub fn merge(new_sarif: &Sarif, old_sarif: &Sarif) -> Sarif {
    let fp_map = build_fingerprint_map(old_sarif);
    let old_result_count = count_old_results(old_sarif);

    let mut merged_runs = Vec::with_capacity(new_sarif.runs.len());

    for run in &new_sarif.runs {
        let counts = count_run_changes(run, &fp_map, old_result_count);
        merged_runs.push(merge_run(run, &fp_map, counts));
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
    counts: (usize, usize, usize),
) -> serde_sarif::sarif::Run {
    let mut new_run = run.clone();

    let Some(results) = &run.results else {
        return new_run;
    };

    let mut new_results = Vec::with_capacity(results.len());

    for result in results {
        new_results.push(merge_result(result, fp_map));
    }

    new_run.results = Some(new_results);

    let (new, updated, closed) = counts;
    new_run.automation_details = Some(serde_sarif::sarif::RunAutomationDetails {
        description: Some(serde_sarif::sarif::Message {
            text: Some(format!(
                "{} new, {} updated and {} closed results.",
                new, updated, closed
            )),
            ..Default::default()
        }),
        ..Default::default()
    });

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_sarif::sarif::Sarif;

    #[test]
    fn merge_returns_new_sarif() {
        let new_sarif: Sarif =
            serde_json::from_str(include_str!("../tests/fixtures/returns_new_new.sarif"))
                .unwrap();
        let old_sarif: Sarif =
            serde_json::from_str(include_str!("../tests/fixtures/returns_new_old.sarif"))
                .unwrap();

        let merged_sarif = merge(&new_sarif, &old_sarif);

        assert_eq!(new_sarif, merged_sarif);
    }

    #[test]
    fn merge_updates_message_from_matching_fingerprint_in_any_result_in_any_run() {
        let new_sarif: Sarif = serde_json::from_str(include_str!(
            "../tests/fixtures/updates_and_counts_new.sarif"
        ))
        .unwrap();

        let old_sarif: Sarif = serde_json::from_str(include_str!(
            "../tests/fixtures/updates_and_counts_old.sarif"
        ))
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

    #[test]
    fn counts_new_updated_closed_results_for_run() {
        let new_sarif: Sarif = serde_json::from_str(include_str!(
            "../tests/fixtures/updates_and_counts_new.sarif"
        ))
        .unwrap();

        let old_sarif: Sarif = serde_json::from_str(include_str!(
            "../tests/fixtures/updates_and_counts_old.sarif"
        ))
        .unwrap();

        let merged_sarif = merge(&new_sarif, &old_sarif);

        assert_eq!(
            "1 new, 1 updated and 1 closed results.",
            merged_sarif.runs[0]
                .automation_details
                .as_ref()
                .unwrap()
                .description
                .as_ref()
                .unwrap()
                .text
                .as_ref()
                .unwrap()
        );
    }

    #[test]
    fn merge_copies_rank_from_matching_fingerprint() {
        let new_sarif: Sarif =
            serde_json::from_str(include_str!("../tests/fixtures/copies_rank_new.sarif"))
                .unwrap();

        let old_sarif: Sarif =
            serde_json::from_str(include_str!("../tests/fixtures/copies_rank_old.sarif"))
                .unwrap();

        let merged = merge(&new_sarif, &old_sarif);

        assert_eq!(Some(5.0), merged.runs[0].results.as_ref().unwrap()[0].rank);
    }

    #[test]
    fn merge_copies_suppressions_from_matching_fingerprint() {
        let new_sarif: Sarif = serde_json::from_str(include_str!(
            "../tests/fixtures/copies_suppressions_new.sarif"
        ))
        .unwrap();

        let old_sarif: Sarif = serde_json::from_str(include_str!(
            "../tests/fixtures/copies_suppressions_old.sarif"
        ))
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
}

