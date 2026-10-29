// LiveSpec traceability: @spec(AC-006)
//! Bounded search over a scripted tree (013 FR-006, AC-006, SC-002).
use super::*;
use crate::destination::default_input::fake_tree::{
    element, FakeElement, FakeTree, Fallible, TestResult,
};
use crate::destination::default_input::fingerprint::{fingerprint_of, SECURE_TEXT_FIELD};

const PID: u32 = 42;

/// A window holding a conversation group and the composer named `name`.
fn conversation_window(composer_name: &str) -> Vec<FakeElement> {
    vec![
        element("AXWindow", None, vec![1, 3]),
        element("AXGroup", Some("Conversation"), vec![2]),
        element("AXStaticText", None, vec![]),
        element("AXTextArea", Some(composer_name), vec![]),
    ]
}
fn fingerprint_of_element(elements: &[FakeElement], id: usize) -> Fallible<FieldFingerprint> {
    Ok(fingerprint_of(&elements[id].facts).map_err(|error| format!("{error:?}"))?)
}

// @spec AC-006: Unique match among the window's elements — .specs/features/013-default-input-per-app/spec.md#ac-006
#[test]
fn a_single_matching_field_is_unique() -> TestResult {
    let elements = conversation_window("Work with Codex");
    let fingerprint = fingerprint_of_element(&elements, 3)?;
    let tree = FakeTree::new(PID, elements);
    let (outcome, stats) = search(&tree, PID, &fingerprint, tree.clock.as_ref());
    assert!(matches!(outcome, SearchOutcome::Unique(_)), "{outcome:?}");
    assert_eq!((stats.visited, stats.matches), (4, 1));
    Ok(())
}

#[test]
fn no_match_and_two_matches_never_prove_a_field() -> TestResult {
    let elements = conversation_window("Work with Codex");
    let mut other = fingerprint_of_element(&elements, 3)?;
    other.name = Some("Search".to_owned());
    let tree = FakeTree::new(PID, elements.clone());
    let (outcome, _) = search(&tree, PID, &other, tree.clock.as_ref());
    assert!(matches!(outcome, SearchOutcome::NotFound), "{outcome:?}");

    let mut twice = elements;
    twice[0].children.push(4);
    twice.push(twice[3].clone());
    let fingerprint = fingerprint_of_element(&twice, 3)?;
    let tree = FakeTree::new(PID, twice);
    let (outcome, stats) = search(&tree, PID, &fingerprint, tree.clock.as_ref());
    assert!(matches!(outcome, SearchOutcome::Several), "{outcome:?}");
    assert_eq!(stats.matches, 2);
    Ok(())
}

#[test]
fn a_password_field_with_the_same_name_is_never_a_match() -> TestResult {
    let mut elements = conversation_window("Secret");
    let fingerprint = fingerprint_of_element(&elements, 3)?;
    elements[3].facts.subrole = Some(SECURE_TEXT_FIELD.to_owned());
    let tree = FakeTree::new(PID, elements);
    let (outcome, _) = search(&tree, PID, &fingerprint, tree.clock.as_ref());
    assert!(matches!(outcome, SearchOutcome::NotFound), "{outcome:?}");
    Ok(())
}

#[test]
fn children_of_static_texts_are_not_searched() -> TestResult {
    let mut elements = conversation_window("Work with Codex");
    let fingerprint = fingerprint_of_element(&elements, 3)?;
    elements[2].children = vec![4];
    elements.push(elements[3].clone());
    let tree = FakeTree::new(PID, elements);
    let (outcome, _) = search(&tree, PID, &fingerprint, tree.clock.as_ref());
    assert!(matches!(outcome, SearchOutcome::Unique(_)), "{outcome:?}");
    Ok(())
}

/// A chain of `length` groups ending with the composer.
fn deep_chain(length: usize) -> Vec<FakeElement> {
    let mut elements: Vec<FakeElement> = (0..length)
        .map(|index| element("AXGroup", None, vec![index + 1]))
        .collect();
    elements.push(element("AXTextArea", Some("Work with Codex"), vec![]));
    elements
}

#[test]
fn the_depth_bound_makes_the_search_incomplete() -> TestResult {
    let within = deep_chain(MAX_DEPTH);
    let fingerprint = fingerprint_of_element(&within, MAX_DEPTH)?;
    let tree = FakeTree::new(PID, within);
    let (outcome, _) = search(&tree, PID, &fingerprint, tree.clock.as_ref());
    assert!(matches!(outcome, SearchOutcome::Unique(_)), "{outcome:?}");
    let beyond = deep_chain(MAX_DEPTH + 1);
    let tree = FakeTree::new(PID, beyond);
    let (outcome, _) = search(&tree, PID, &fingerprint, tree.clock.as_ref());
    assert!(
        matches!(outcome, SearchOutcome::Incomplete(Incomplete::Depth)),
        "{outcome:?}"
    );
    Ok(())
}

#[test]
fn the_node_and_time_bounds_make_the_search_incomplete() -> TestResult {
    let wide: Vec<usize> = (1..=MAX_NODES).collect();
    let mut elements = vec![element("AXWindow", None, wide)];
    elements.extend((0..MAX_NODES).map(|_| element("AXGroup", None, vec![])));
    let fingerprint = FieldFingerprint {
        role: "AXTextArea".to_owned(),
        subrole: None,
        name: Some("Work with Codex".to_owned()),
        placeholder: None,
        dom_id: None,
    };
    let tree = FakeTree::new(PID, elements.clone());
    let (outcome, stats) = search(&tree, PID, &fingerprint, tree.clock.as_ref());
    assert!(
        matches!(outcome, SearchOutcome::Incomplete(Incomplete::Nodes)),
        "{outcome:?}"
    );
    assert_eq!(stats.visited, MAX_NODES);

    let slow = FakeTree::new(PID, elements);
    slow.lock().read_cost = Duration::from_millis(5);
    let (outcome, stats) = search(&slow, PID, &fingerprint, slow.clock.as_ref());
    assert!(
        matches!(outcome, SearchOutcome::Incomplete(Incomplete::Budget)),
        "{outcome:?}"
    );
    assert!(stats.elapsed <= SEARCH_BUDGET);
    Ok(())
}

#[test]
fn a_timeout_is_incomplete_and_an_unreadable_element_is_skipped() -> TestResult {
    let mut elements = conversation_window("Work with Codex");
    let fingerprint = fingerprint_of_element(&elements, 3)?;
    elements[1].read_error = Some(AxQueryError::Failed);
    let tree = FakeTree::new(PID, elements.clone());
    let (outcome, _) = search(&tree, PID, &fingerprint, tree.clock.as_ref());
    assert!(matches!(outcome, SearchOutcome::Unique(_)), "{outcome:?}");
    elements[1].read_error = Some(AxQueryError::Timeout);
    let tree = FakeTree::new(PID, elements);
    let (outcome, _) = search(&tree, PID, &fingerprint, tree.clock.as_ref());
    assert!(
        matches!(outcome, SearchOutcome::Incomplete(Incomplete::Budget)),
        "{outcome:?}"
    );
    Ok(())
}

#[test]
fn another_process_or_no_window_reads_nothing() -> TestResult {
    let elements = conversation_window("Work with Codex");
    let fingerprint = fingerprint_of_element(&elements, 3)?;
    let tree = FakeTree::new(PID, elements);
    let (outcome, stats) = search(&tree, PID + 1, &fingerprint, tree.clock.as_ref());
    assert!(
        matches!(outcome, SearchOutcome::Incomplete(Incomplete::NoWindow)),
        "{outcome:?}"
    );
    assert_eq!((stats.visited, tree.lock().reads), (0, 0));
    Ok(())
}
