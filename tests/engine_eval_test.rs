use flagsmith_flag_engine::engine::get_evaluation_result;
use flagsmith_flag_engine::engine_eval::{EngineEvaluationContext, EvaluationResult};
use json_comments::StripComments;
use rstest::*;
use serde::Deserialize;
use std::fs;
use std::io::Read;

#[derive(Deserialize)]
struct TestCase {
    context: EngineEvaluationContext,
    result: EvaluationResult,
}

#[rstest]
fn test_engine_evaluation() {
    // Get all test files
    let test_dir = "tests/engine_tests/engine-test-data/test_cases";
    let test_files = fs::read_dir(test_dir).expect("Failed to read test directory");

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

        // Deserialize the context and the expected result, keeping segments in context order
        let TestCase {
            context,
            result: expected,
        } = serde_json::from_str(&json_string)
            .unwrap_or_else(|e| panic!("Failed to deserialize {}: {:?}", test_name, e));

        // Get the evaluation result
        let result = get_evaluation_result(&context);

        // Compare results - panic immediately on mismatch
        assert_eq!(
            result.flags, expected.flags,
            "Flags mismatch in {}",
            test_name
        );
        assert_eq!(
            result.segments, expected.segments,
            "Segments mismatch in {}",
            test_name
        );
    }
}
