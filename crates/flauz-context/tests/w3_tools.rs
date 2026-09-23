//! The dynamic tool-exposure laws (Wave 3, ORCH-002): excluded tools are
//! NAMED (never silently dropped), schema-format conversion honors the
//! per-profile tool-schema handling, and the exposure is deterministic
//! and order-independent.

use std::fmt;

use flauz_context::fakes::{fake_capability_admissions, fake_tool_catalog};
use flauz_context::profile::{ModelContextProfile, MultimodalBehavior, ToolSchemaHandling};
use flauz_context::{
    ActorRef, CapabilityAdmission, ModelRef, Timestamp, ToolDefinition, ToolExposure,
    ToolPresentation, compute_tool_exposure, validate_capability_key,
};

fn test_ok<T, E: fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{error}"),
    }
}

const MODEL_A: &str = "model_01J8ZQ5V8K3T2B7N6X4R9DQPRE";

fn profile_with(tool_schema: ToolSchemaHandling) -> ModelContextProfile {
    test_ok(ModelContextProfile::new(
        test_ok(ModelRef::parse(MODEL_A)),
        200_000,
        MultimodalBehavior::ImageInput,
        tool_schema,
        test_ok(ActorRef::system("flauz-fake")),
        test_ok(Timestamp::parse("2026-09-21T13:45:00Z")),
    ))
}

/// Exclusions are NAMED, never silently dropped (acceptance criterion
/// 4): every tool in the input appears exactly once across the exposed
/// and excluded lists; a gap-blocked tool carries the named gap reasons
/// of its capability resolution; a tool with no resolution recorded at
/// all carries the explicit no-resolution reason.
#[test]
fn exposure_names_its_exclusions_never_silently_drops() {
    let profile = profile_with(ToolSchemaHandling::SummariesWithLazySchemas);
    let tools = test_ok(fake_tool_catalog());
    let admissions = test_ok(fake_capability_admissions());

    let exposure = test_ok(compute_tool_exposure(&profile, &tools, &admissions));

    // Every input tool is accounted for exactly once.
    assert_eq!(exposure.exposed.len(), 2);
    assert_eq!(exposure.excluded.len(), 2);
    let mut names: Vec<&str> = exposure
        .exposed
        .iter()
        .map(|tool| tool.name.as_str())
        .chain(exposure.excluded.iter().map(|tool| tool.name.as_str()))
        .collect();
    names.sort_unstable();
    assert_eq!(
        names,
        vec!["browse_web", "read_file", "run_command", "view_screen"]
    );

    // The gap-blocked tool is excluded with the honest named-gap reason.
    let browse = exposure
        .excluded
        .iter()
        .find(|tool| tool.name == "browse_web")
        .unwrap_or_else(|| panic!("browse_web is excluded"));
    assert_eq!(browse.capability, "browser.navigation");
    assert_eq!(
        browse.reason,
        "model: the model in use does not offer it; environment: the attached environment \
         does not offer this capability"
    );

    // The unresolved tool is excluded with the explicit no-resolution
    // reason — the silent fall-through the kernel forbids.
    let screen = exposure
        .excluded
        .iter()
        .find(|tool| tool.name == "view_screen")
        .unwrap_or_else(|| panic!("view_screen is excluded"));
    assert_eq!(screen.capability, "computer.screen");
    assert_eq!(
        screen.reason,
        "no capability resolution is recorded for computer.screen"
    );

    test_ok(exposure.validate());
}

/// Schema-format conversion per profile (acceptance criterion 4): the
/// three frozen ToolSchemaHandling modes produce full-schema, summary and
/// lazy name-only presentations respectively.
#[test]
fn per_profile_schema_handling_is_honored() {
    let tools = test_ok(fake_tool_catalog());
    let admissions = test_ok(fake_capability_admissions());

    let full = test_ok(compute_tool_exposure(
        &profile_with(ToolSchemaHandling::InlineFullSchemas),
        &tools,
        &admissions,
    ));
    for tool in &full.exposed {
        assert!(
            matches!(&tool.presentation, ToolPresentation::Full { schema } if schema.contains("\"type\":\"object\""))
        );
    }

    let summaries = test_ok(compute_tool_exposure(
        &profile_with(ToolSchemaHandling::SummariesWithLazySchemas),
        &tools,
        &admissions,
    ));
    for tool in &summaries.exposed {
        assert!(
            matches!(&tool.presentation, ToolPresentation::Summary { summary } if !summary.is_empty())
        );
    }
    // The full schema is NOT inlined in summary mode.
    let serialized = test_ok(serde_json::to_string(&summaries));
    assert!(!serialized.contains("required"));

    let lazy = test_ok(compute_tool_exposure(
        &profile_with(ToolSchemaHandling::LazyPerTool),
        &tools,
        &admissions,
    ));
    for tool in &lazy.exposed {
        assert!(matches!(tool.presentation, ToolPresentation::Lazy));
    }
    assert_eq!(lazy.excluded, summaries.excluded, "exclusions are stable");
}

/// The exposure is deterministic and order-independent: recomputing
/// yields the identical record, and reversing the tool and admission
/// lists yields the identical record too (output sorted by tool name).
#[test]
fn exposure_is_deterministic_and_order_independent() {
    let profile = profile_with(ToolSchemaHandling::SummariesWithLazySchemas);
    let mut tools = test_ok(fake_tool_catalog());
    let mut admissions = test_ok(fake_capability_admissions());

    let first = test_ok(compute_tool_exposure(&profile, &tools, &admissions));
    let second = test_ok(compute_tool_exposure(&profile, &tools, &admissions));
    assert_eq!(first, second);

    tools.reverse();
    admissions.reverse();
    let reversed = test_ok(compute_tool_exposure(&profile, &tools, &admissions));
    assert_eq!(first, reversed);

    // Canonical round-trip; unknown fields are rejected on read.
    let serialized = test_ok(serde_json::to_string(&first));
    let reloaded: ToolExposure = test_ok(serde_json::from_str(&serialized));
    assert_eq!(reloaded, first);
    assert!(
        serde_json::from_str::<ToolExposure>(&serialized.replace(
            "\"tool_schema\":\"summaries_with_lazy_schemas\"",
            "\"tool_schema\":\"summaries_with_lazy_schemas\",\"totally_unknown\":true"
        ))
        .is_err(),
        "unknown fields must be rejected"
    );
    test_ok(reloaded.validate());
}

/// Input-shape violations are rejected honestly: duplicate tool names,
/// duplicate capability admissions, an unresolvable tool count, and
/// inconsistent admissions (the silent-fall-through shape).
#[test]
fn exposure_input_violations_are_rejected() {
    let profile = profile_with(ToolSchemaHandling::LazyPerTool);
    let tools = test_ok(fake_tool_catalog());
    let admissions = test_ok(fake_capability_admissions());

    // Duplicate tool names.
    let mut duplicated = tools.clone();
    duplicated[1].name = "read_file".to_owned();
    assert!(compute_tool_exposure(&profile, &duplicated, &admissions).is_err());

    // Duplicate capability admissions.
    let mut twice = admissions.clone();
    twice.push(admissions[0].clone());
    assert!(compute_tool_exposure(&profile, &tools, &twice).is_err());

    // The silent-fall-through shape is an invalid admission.
    assert!(
        CapabilityAdmission::new(
            "browser.navigation",
            true,
            vec![test_ok(flauz_context::AdmissionGap::new(
                "model",
                "the model in use does not offer it"
            ))]
        )
        .is_err()
    );

    // An admission with a non-frozen dimension name: the gap constructor
    // itself rejects it, so the malformed admission can never exist.
    assert!(flauz_context::AdmissionGap::new("mood", "the model is not feeling it").is_err());
}

/// The frozen capability-key grammar and the five frozen dimension names
/// are re-pinned locally (the CAP-001 frozen-format pattern: this crate
/// does not import flauz-cap; the grammar vectors assert the re-pin is
/// faithful).
#[test]
fn the_frozen_grammars_are_repinned() {
    for valid in [
        "terminal",
        "browser.input",
        "browser.navigation",
        "filesystem.read",
        "filesystem.write",
        "computer.screen",
        "flauz.tool.read_file",
        "a.b.c_d",
    ] {
        test_ok(validate_capability_key(valid));
    }
    for invalid in [
        "",
        "Terminal",
        "terminal.",
        ".terminal",
        "terminal..input",
        "browser..input",
        "browser.Input",
        "9browser",
        "browser-input",
    ] {
        assert!(
            validate_capability_key(invalid).is_err(),
            "{invalid:?} must fail the frozen capability-key grammar"
        );
    }
    for dimension in flauz_context::tools::CAPABILITY_DIMENSIONS {
        test_ok(flauz_context::AdmissionGap::new(
            dimension,
            "the dimension does not admit it",
        ));
    }
    assert_eq!(
        flauz_context::tools::CAPABILITY_DIMENSIONS.len(),
        5,
        "five frozen admission dimensions"
    );

    // Tool definitions validate the grammar at construction.
    assert!(ToolDefinition::new("read_file", "Terminal", "{}", "summary").is_err());
    assert!(ToolDefinition::new("read_file", "filesystem.read", "{}", "summary").is_ok());
}
