use criterion::{criterion_group, criterion_main, Criterion};
use flagsmith_flag_engine::engine::get_evaluation_result;
use flagsmith_flag_engine::engine_eval::EngineEvaluationContext;
use json_comments::StripComments;
use serde_json::{json, Value};
use std::hint::black_box;
use std::path::{Path, PathBuf};

const TEST_CASES_DIR: &str = "tests/engine_tests/engine-test-data/test_cases";

/// The contexts flagsmith-engine benchmarks, so that both engines measure the same workload
const REPRESENTATIVE_TEST_CASES: [&str; 5] = [
    "test_0cfd0d72-4de4-4ed7-9cfb-d80dc3dacead__default.json",
    "test_1bde8445-ca19-4bda-a9d5-3543a800fc0f__context_values.json",
    "test_in_condition_json_array_format__should_match.jsonc",
    "test_in_condition_numeric_comma_separated__should_match.jsonc",
    "test_in_condition_array_matching_value__should_match.jsonc",
];

const FEATURE_COUNT: usize = 200;
const SEGMENT_COUNT: usize = 50;
const TRAIT_COUNT: usize = 20;

fn read_test_case_context(path: &Path) -> Value {
    let test_case = std::fs::read(path).unwrap();
    let test_case: Value = serde_json::from_reader(StripComments::new(test_case.as_slice()))
        .unwrap_or_else(|e| panic!("Failed to parse {}: {e}", path.display()));
    test_case["context"].clone()
}

fn test_cases_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(TEST_CASES_DIR)
}

fn representative_contexts() -> Vec<EngineEvaluationContext> {
    REPRESENTATIVE_TEST_CASES
        .iter()
        .map(|name| {
            serde_json::from_value(read_test_case_context(&test_cases_dir().join(name))).unwrap()
        })
        .collect()
}

/// Every engine-test-data context, except those reading flags, which engines without
/// flag dependencies evaluate differently
fn corpus_contexts_without_flag_dependencies() -> Vec<EngineEvaluationContext> {
    let mut paths: Vec<_> = std::fs::read_dir(test_cases_dir())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("json" | "jsonc")
            )
        })
        .collect();
    paths.sort();
    paths
        .iter()
        .map(|path| read_test_case_context(path))
        .filter(|context| !context.to_string().contains("$.flags"))
        .map(|context| serde_json::from_value(context).unwrap())
        .collect()
}

/// An environment of `FEATURE_COUNT` features, a tenth multivariate, and `SEGMENT_COUNT`
/// segments, each overriding four of the first 151 features on conditions returned by `rules`
fn large_context(identity: bool, rules: impl Fn(usize) -> Value) -> EngineEvaluationContext {
    let features: serde_json::Map<String, Value> = (0..FEATURE_COUNT)
        .map(|i| {
            let variants = if i % 10 == 0 {
                json!([
                    {"key": format!("{i}-a"), "value": "a", "weight": 30.0, "priority": 1},
                    {"key": format!("{i}-b"), "value": "b", "weight": 30.0, "priority": 2},
                    {"key": format!("{i}-c"), "value": "c", "weight": 30.0, "priority": 3}
                ])
            } else {
                json!([])
            };
            (
                format!("feature_{i}"),
                json!({
                    "key": i.to_string(),
                    "name": format!("feature_{i}"),
                    "enabled": i % 2 == 0,
                    "value": format!("value_{i}"),
                    "variants": variants,
                    "metadata": {"feature_id": i}
                }),
            )
        })
        .collect();
    let segments: serde_json::Map<String, Value> = (0..SEGMENT_COUNT)
        .map(|i| {
            let overrides: Vec<Value> = (0..4)
                .map(|j| {
                    let feature = i * 3 + j;
                    json!({
                        "key": feature.to_string(),
                        "name": format!("feature_{feature}"),
                        "enabled": true,
                        "value": format!("override_{i}"),
                        "priority": i
                    })
                })
                .collect();
            (
                i.to_string(),
                json!({
                    "key": i.to_string(),
                    "name": format!("segment_{i}"),
                    "rules": rules(i),
                    "overrides": overrides
                }),
            )
        })
        .collect();
    let traits: serde_json::Map<String, Value> = (0..TRAIT_COUNT)
        .map(|i| (format!("trait_{i}"), json!(i)))
        .collect();

    let mut context = json!({
        "environment": {"key": "environment_key", "name": "Environment"},
        "features": features,
        "segments": segments
    });
    if identity {
        context["identity"] = json!({"identifier": "user@example.com", "traits": traits});
    }
    serde_json::from_value(context).unwrap()
}

/// Trait conditions of every common operator, a fifth of segments matching
fn trait_rules(segment: usize) -> Value {
    let trait_name = format!("trait_{}", segment % TRAIT_COUNT);
    json!([{
        "type": "ALL",
        "rules": [{
            "type": "ANY",
            "conditions": [
                {"operator": "EQUAL", "property": trait_name, "value": (segment % 5).to_string()},
                {"operator": "IN", "property": trait_name, "value": "100,101,102"}
            ]
        }, {
            "type": "ALL",
            "conditions": [
                {"operator": "GREATER_THAN_INCLUSIVE", "property": format!("$.identity.traits.{trait_name}"), "value": "0"},
                {"operator": "PERCENTAGE_SPLIT", "property": "", "value": "90"}
            ]
        }]
    }])
}

/// Conditions on the flag overridden by the previous segment, so that features form one long
/// dependency chain
fn flag_chain_rules(segment: usize) -> Value {
    let rules = trait_rules(segment);
    if segment == 0 {
        return rules;
    }
    let prerequisite = (segment - 1) * 3;
    json!([rules[0], {
        "type": "ALL",
        "conditions": [{
            "operator": "EQUAL",
            "property": format!("$.flags.feature_{prerequisite}.enabled"),
            "value": "true"
        }]
    }])
}

/// Conditions on one of five prerequisite flags, which no segment overrides
fn flag_fan_out_rules(segment: usize) -> Value {
    let rules = trait_rules(segment);
    let prerequisite = FEATURE_COUNT - 1 - segment % 5;
    json!([rules[0], {
        "type": "ALL",
        "conditions": [{
            "operator": "EQUAL",
            "property": format!("$.flags['feature_{prerequisite}'].value"),
            "value": format!("value_{prerequisite}")
        }]
    }])
}

fn bench_contexts(c: &mut Criterion, name: &str, contexts: &[EngineEvaluationContext]) {
    c.bench_function(name, |b| {
        b.iter(|| {
            for context in contexts {
                black_box(get_evaluation_result(black_box(context)));
            }
        })
    });
}

fn engine_test_data(c: &mut Criterion) {
    bench_contexts(
        c,
        "engine_test_data/representative",
        &representative_contexts(),
    );
    bench_contexts(
        c,
        "engine_test_data/corpus_without_flag_dependencies",
        &corpus_contexts_without_flag_dependencies(),
    );
}

fn large_environment(c: &mut Criterion) {
    bench_contexts(
        c,
        "large_environment/environment",
        &[large_context(false, trait_rules)],
    );
    bench_contexts(
        c,
        "large_environment/identity",
        &[large_context(true, trait_rules)],
    );
    bench_contexts(
        c,
        "large_environment/identity_with_flag_chain",
        &[large_context(true, flag_chain_rules)],
    );
    bench_contexts(
        c,
        "large_environment/identity_with_flag_fan_out",
        &[large_context(true, flag_fan_out_rules)],
    );
}

criterion_group!(benches, engine_test_data, large_environment);
criterion_main!(benches);
