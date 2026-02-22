use rust_yaml::{Value, Yaml, YamlConfig};

/// Bug 1 (parser): After a flow value ([], {}, [{...}]) at an indentation level,
/// nested block mappings that follow at the same level or higher are mis-parsed.
/// The parser lost its block context after processing flow collections.
///
/// Bug 2 (emitter): Empty collections emitted with spurious newline:
///   `list:\n[]` instead of `list: []`
///
/// Bug 3 (emitter): Added emit_anchors option to allow disabling
///   automatic anchor/alias generation for shared values.

#[test]
fn test_flow_seq_then_block_mapping_same_level() {
    let yaml = Yaml::new();
    let input = r#"
key1: []
key2: value2
"#;
    let result = yaml.load_str(input).unwrap();
    println!("Test 1 result: {:?}", result);

    if let Value::Mapping(ref map) = result {
        assert_eq!(
            map.get(&Value::String("key1".to_string())),
            Some(&Value::Sequence(vec![])),
            "key1 should be empty sequence"
        );
        assert_eq!(
            map.get(&Value::String("key2".to_string())),
            Some(&Value::String("value2".to_string())),
            "key2 should be 'value2'"
        );
        assert_eq!(map.len(), 2, "should have exactly 2 keys");
    } else {
        panic!("Expected a mapping, got: {:?}", result);
    }
}

#[test]
fn test_flow_map_then_block_mapping_same_level() {
    let yaml = Yaml::new();
    let input = r#"
key1: {a: b}
key2: value2
"#;
    let result = yaml.load_str(input).unwrap();
    println!("Test 2 result: {:?}", result);

    if let Value::Mapping(ref map) = result {
        assert_eq!(map.len(), 2, "should have exactly 2 keys");
        assert!(
            map.contains_key(&Value::String("key2".to_string())),
            "key2 must exist"
        );
        assert_eq!(
            map.get(&Value::String("key2".to_string())),
            Some(&Value::String("value2".to_string())),
        );
    } else {
        panic!("Expected a mapping, got: {:?}", result);
    }
}

#[test]
fn test_nested_flow_then_block_mapping() {
    let yaml = Yaml::new();
    let input = r#"
parent:
  child1: []
  child2: value2
"#;
    let result = yaml.load_str(input).unwrap();
    println!("Test 3 result: {:?}", result);

    if let Value::Mapping(ref map) = result {
        let parent = map.get(&Value::String("parent".to_string())).unwrap();
        if let Value::Mapping(ref inner) = parent {
            assert_eq!(inner.len(), 2, "parent should have 2 children");
            assert_eq!(
                inner.get(&Value::String("child1".to_string())),
                Some(&Value::Sequence(vec![])),
            );
            assert_eq!(
                inner.get(&Value::String("child2".to_string())),
                Some(&Value::String("value2".to_string())),
            );
        } else {
            panic!("parent should be a mapping, got: {:?}", parent);
        }
    } else {
        panic!("Expected a mapping, got: {:?}", result);
    }
}

#[test]
fn test_flow_seq_with_objects_then_block() {
    let yaml = Yaml::new();
    let input = r#"
key1: [{a: b}]
key2: value2
"#;
    let result = yaml.load_str(input).unwrap();
    println!("Test 4 result: {:?}", result);

    if let Value::Mapping(ref map) = result {
        assert_eq!(map.len(), 2, "should have exactly 2 keys");
        assert!(
            map.contains_key(&Value::String("key1".to_string())),
            "key1 must exist"
        );
        assert!(
            map.contains_key(&Value::String("key2".to_string())),
            "key2 must exist"
        );
    } else {
        panic!("Expected a mapping, got: {:?}", result);
    }
}

#[test]
fn test_kubernetes_like_structure() {
    let yaml = Yaml::new();
    let input = r#"
metadata:
  name: test
  labels: {}
spec:
  replicas: 3
"#;
    let result = yaml.load_str(input).unwrap();
    println!("Test 5 result: {:?}", result);

    if let Value::Mapping(ref map) = result {
        assert_eq!(map.len(), 2, "should have metadata and spec");
        assert!(
            map.contains_key(&Value::String("metadata".to_string())),
            "metadata must exist"
        );
        assert!(
            map.contains_key(&Value::String("spec".to_string())),
            "spec must exist"
        );

        // Verify spec structure
        if let Some(Value::Mapping(ref spec)) = map.get(&Value::String("spec".to_string())) {
            assert_eq!(
                spec.get(&Value::String("replicas".to_string())),
                Some(&Value::Int(3)),
            );
        } else {
            panic!("spec should be a mapping");
        }
    } else {
        panic!("Expected a mapping, got: {:?}", result);
    }
}

#[test]
fn test_multiple_flow_values_then_block() {
    let yaml = Yaml::new();
    let input = r#"
a: []
b: {}
c: [{x: 1}]
d: normal
"#;
    let result = yaml.load_str(input).unwrap();
    println!("Test 6 result: {:?}", result);

    if let Value::Mapping(ref map) = result {
        assert_eq!(map.len(), 4, "should have 4 keys");
        assert_eq!(
            map.get(&Value::String("d".to_string())),
            Some(&Value::String("normal".to_string())),
        );
    } else {
        panic!("Expected a mapping, got: {:?}", result);
    }
}

// --- Emitter bug tests: empty collections should be inline ---

#[test]
fn test_emit_empty_sequence_inline() {
    let yaml = Yaml::new();
    let input = r#"
key1: []
key2: value2
"#;
    let parsed = yaml.load_str(input).unwrap();
    let output = yaml.dump_str(&parsed).unwrap();

    assert!(
        output.contains("key1: []"),
        "Empty sequence should be inline, got:\n{}",
        output
    );
    assert!(
        !output.contains("key1: \n"),
        "Should not have newline before empty sequence"
    );
}

#[test]
fn test_emit_empty_mapping_inline() {
    let yaml = Yaml::new();
    let input = r#"
key1: {}
key2: value2
"#;
    let parsed = yaml.load_str(input).unwrap();
    let output = yaml.dump_str(&parsed).unwrap();

    assert!(
        output.contains("key1: {}"),
        "Empty mapping should be inline, got:\n{}",
        output
    );
    assert!(
        !output.contains("key1: \n"),
        "Should not have newline before empty mapping"
    );
}

#[test]
fn test_emit_kubernetes_roundtrip() {
    let yaml = Yaml::new();
    let input = r#"
metadata:
  name: test
  labels: {}
spec:
  replicas: 3
"#;
    let parsed = yaml.load_str(input).unwrap();
    let output = yaml.dump_str(&parsed).unwrap();

    assert!(
        output.contains("labels: {}"),
        "Empty labels should be inline, got:\n{}",
        output
    );

    // Verify round-trip correctness
    let reparsed = yaml.load_str(&output).unwrap();
    assert_eq!(parsed, reparsed, "Round-trip should preserve structure");
}

// --- Emitter formatting tests: no trailing space, inline mapping in sequences ---

#[test]
fn test_no_trailing_space_after_colon() {
    let yaml = Yaml::new();
    let input = "parent:\n  child:\n    key: value\n";
    let parsed = yaml.load_str(input).unwrap();
    let output = yaml.dump_str(&parsed).unwrap();

    // No line should end with ": " (trailing space)
    for line in output.lines() {
        assert!(
            !line.ends_with(": "),
            "Line should not end with trailing space after colon: {:?}",
            line
        );
    }
}

#[test]
fn test_mapping_inline_with_sequence_dash() {
    let yaml = Yaml::new();
    let input = r#"
items:
  - name: foo
    value: bar
  - name: baz
    value: qux
"#;
    let parsed = yaml.load_str(input).unwrap();
    let output = yaml.dump_str(&parsed).unwrap();

    // Mapping entries should start on the same line as "- "
    assert!(
        output.contains("- name: foo"),
        "First mapping key should be inline with '- ', got:\n{}",
        output
    );
    assert!(
        output.contains("- name: baz"),
        "Second mapping key should be inline with '- ', got:\n{}",
        output
    );

    // Round-trip correctness
    let reparsed = yaml.load_str(&output).unwrap();
    assert_eq!(parsed, reparsed, "Round-trip should preserve structure");
}

#[test]
fn test_crd_like_structure() {
    let yaml = Yaml::new();
    let input = r#"
versions:
  - additionalPrinterColumns:
      - description: Update schedule
        name: schedule
        type: string
      - description: Last update date
        name: last_updated
        type: date
"#;
    let parsed = yaml.load_str(input).unwrap();
    let output = yaml.dump_str(&parsed).unwrap();

    // No trailing spaces on any line
    for line in output.lines() {
        assert!(
            !line.ends_with(' '),
            "Line should not end with trailing space: {:?}",
            line
        );
    }

    // Mappings should be inline with "- "
    assert!(
        output.contains("- additionalPrinterColumns:"),
        "Should have '- additionalPrinterColumns:', got:\n{}",
        output
    );
    assert!(
        output.contains("- description: Update schedule"),
        "Should have '- description: Update schedule', got:\n{}",
        output
    );

    // Round-trip correctness
    let reparsed = yaml.load_str(&output).unwrap();
    assert_eq!(parsed, reparsed, "Round-trip should preserve structure");
}

// --- Anchor/alias emission option tests ---

#[test]
fn test_anchors_emitted_by_default() {
    let yaml = Yaml::new();

    // Build a structure with shared (identical) values
    let mut shared_map = indexmap::IndexMap::new();
    shared_map.insert(
        Value::String("x".to_string()),
        Value::String("shared".to_string()),
    );
    let shared = Value::Mapping(shared_map);

    let mut root = indexmap::IndexMap::new();
    root.insert(Value::String("a".to_string()), shared.clone());
    root.insert(Value::String("b".to_string()), shared);

    let value = Value::Mapping(root);
    let output = yaml.dump_str(&value).unwrap();

    assert!(
        output.contains('&'),
        "Should contain anchor marker by default, got:\n{}",
        output
    );
    assert!(
        output.contains('*'),
        "Should contain alias marker by default, got:\n{}",
        output
    );
}

#[test]
fn test_no_anchors_when_disabled() {
    let config = YamlConfig {
        emit_anchors: false,
        ..Default::default()
    };
    let yaml = Yaml::with_config(config);

    // Build a structure with shared (identical) values
    let mut shared_map = indexmap::IndexMap::new();
    shared_map.insert(
        Value::String("x".to_string()),
        Value::String("shared".to_string()),
    );
    let shared = Value::Mapping(shared_map);

    let mut root = indexmap::IndexMap::new();
    root.insert(Value::String("a".to_string()), shared.clone());
    root.insert(Value::String("b".to_string()), shared);

    let value = Value::Mapping(root);
    let output = yaml.dump_str(&value).unwrap();

    assert!(
        !output.contains('&'),
        "Should not contain anchor markers when disabled, got:\n{}",
        output
    );
    assert!(
        !output.contains('*'),
        "Should not contain alias markers when disabled, got:\n{}",
        output
    );
}

#[test]
fn test_no_anchors_without_shared_values() {
    let yaml = Yaml::new();

    let input = "key1: value1\nkey2: value2\n";
    let parsed = yaml.load_str(input).unwrap();
    let output = yaml.dump_str(&parsed).unwrap();

    // Even with emit_anchors enabled (default), no anchors if there are no shared values
    assert!(
        !output.contains('&'),
        "Should not have anchors when no shared values, got:\n{}",
        output
    );
}
