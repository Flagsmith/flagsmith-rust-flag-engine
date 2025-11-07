use flagsmith_flag_engine::engine::get_evaluation_result;
use flagsmith_flag_engine::engine_eval::{EngineEvaluationContext, EvaluationResult};
use json_comments::StripComments;
use rstest::*;
use serde_json;
use std::fs;
use std::io::Read;

#[rstest]
fn test_engine_evaluation() {
    // Get all test files
    let test_dir = "tests/engine_tests/engine-test-data/test_cases";
    let test_files = fs::read_dir(test_dir).expect("Failed to read test directory");

    let mut test_count = 0;
    let mut passed = 0;
    let mut failed = 0;
    let mut failed_tests = Vec::new();

    for entry in test_files {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        // Only process JSON and JSONC files
        let extension = path.extension().and_then(|s| s.to_str());
        if extension != Some("json") && extension != Some("jsonc") {
            continue;
        }

        let test_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        test_count += 1;

        // Read the test file
        let test_data =
            fs::read_to_string(&path).expect(&format!("Failed to read test file: {:?}", path));

        // Strip comments if it's a JSONC file
        let json_string = if extension == Some("jsonc") {
            let mut stripped = String::new();
            StripComments::new(test_data.as_bytes())
                .read_to_string(&mut stripped)
                .expect(&format!("Failed to strip comments from: {}", test_name));
            stripped
        } else {
            test_data
        };

        // Parse the JSON
        let test_json: serde_json::Value = serde_json::from_str(&json_string)
            .expect(&format!("Failed to parse JSON for: {}", test_name));

        // Deserialize the context
        let context_result: Result<EngineEvaluationContext, _> =
            serde_json::from_value(test_json["context"].clone());

        if context_result.is_err() {
            println!(
                "FAIL {}: Failed to deserialize context: {:?}",
                test_name,
                context_result.err()
            );
            failed += 1;
            failed_tests.push(test_name.to_string());
            continue;
        }

        let context = context_result.unwrap();

        // Get the evaluation result
        let result = get_evaluation_result(&context);

        // Deserialize the expected result
        let expected_result: Result<EvaluationResult, _> =
            serde_json::from_value(test_json["result"].clone());

        if expected_result.is_err() {
            println!(
                "FAIL {}: Failed to deserialize expected result: {:?}",
                test_name,
                expected_result.err()
            );
            failed += 1;
            failed_tests.push(test_name.to_string());
            continue;
        }

        let expected = expected_result.unwrap();

        // Compare results
        if compare_evaluation_results(&result, &expected, test_name) {
            passed += 1;
            println!("PASS {}", test_name);
        } else {
            failed += 1;
            failed_tests.push(test_name.to_string());
        }
    }

    // Print summary
    println!("\n========== TEST SUMMARY ==========");
    println!("Total tests: {}", test_count);
    println!("Passed: {} ({}%)", passed, (passed * 100) / test_count);
    println!("Failed: {} ({}%)", failed, (failed * 100) / test_count);

    if !failed_tests.is_empty() {
        println!("\nFailed tests:");
        for test in &failed_tests {
            println!("  - {}", test);
        }
    }

    println!("==================================\n");

    // Assert that all tests passed
    assert_eq!(failed, 0, "{} out of {} tests failed", failed, test_count);
}

fn compare_evaluation_results(
    result: &EvaluationResult,
    expected: &EvaluationResult,
    test_name: &str,
) -> bool {
    let mut success = true;

    // Compare flags
    if result.flags.len() != expected.flags.len() {
        println!(
            "FAIL {}: Flag count mismatch - got {}, expected {}",
            test_name,
            result.flags.len(),
            expected.flags.len()
        );
        success = false;
    }

    for (flag_name, expected_flag) in &expected.flags {
        match result.flags.get(flag_name) {
            None => {
                println!("FAIL {}: Missing flag: {}", test_name, flag_name);
                success = false;
            }
            Some(actual_flag) => {
                if actual_flag.enabled != expected_flag.enabled {
                    println!(
                        "FAIL {}: Flag '{}' enabled mismatch - got {}, expected {}",
                        test_name, flag_name, actual_flag.enabled, expected_flag.enabled
                    );
                    success = false;
                }

                if actual_flag.value != expected_flag.value {
                    println!(
                        "FAIL {}: Flag '{}' value mismatch - got {:?}, expected {:?}",
                        test_name, flag_name, actual_flag.value, expected_flag.value
                    );
                    success = false;
                }

                if actual_flag.reason != expected_flag.reason {
                    println!(
                        "FAIL {}: Flag '{}' reason mismatch - got '{}', expected '{}'",
                        test_name, flag_name, actual_flag.reason, expected_flag.reason
                    );
                    success = false;
                }
            }
        }
    }

    // Compare segments
    if result.segments.len() != expected.segments.len() {
        println!(
            "FAIL {}: Segment count mismatch - got {}, expected {}",
            test_name,
            result.segments.len(),
            expected.segments.len()
        );
        success = false;
    }

    success
}
