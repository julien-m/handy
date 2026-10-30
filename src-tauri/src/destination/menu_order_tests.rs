// LiveSpec traceability: @spec(AC-003)
//! Order of the badge menu (012 FR-002): favorites running or not, ordinary applications,
//! duplicates and the frontmost application, then the re-order after a toggle.
use super::*;

fn app(pid: u32, bundle: &str) -> RunningApp {
    RunningApp {
        pid,
        identity: ApplicationIdentity::BundleId(bundle.to_owned()),
        display_name: bundle.to_uppercase(),
        icon_ref: None,
    }
}
fn favorite(bundle: &str) -> FavoriteApp {
    FavoriteApp {
        identity: ApplicationIdentity::BundleId(bundle.to_owned()),
        display_name: format!("Saved {bundle}"),
    }
}
fn names(rows: &[MenuTarget]) -> Vec<String> {
    rows.iter().map(|row| row.display_name.clone()).collect()
}
fn bundles(rows: &[MenuTarget]) -> Vec<String> {
    rows.iter()
        .map(|row| super::super::favorites::identity_value(&row.identity))
        .collect()
}

// @spec FR-002: Pure bottom-up menu order — .specs/features/012-destination-favorites/spec.md#fr-002
#[test]
fn without_favorites_the_frontmost_comes_first_then_the_others() {
    let rows = order_menu(
        Some(app(1, "comet")),
        vec![app(2, "chatgpt"), app(3, "notes")],
        &[],
    );
    assert_eq!(bundles(&rows), ["comet", "chatgpt", "notes"]);
    assert!(rows[0].frontmost && !rows[1].frontmost);
    assert!(rows.iter().all(|row| !row.favorite && row.pid.is_some()));
}

#[test]
fn favorites_come_last_in_stored_order_newest_nearest_the_badge() {
    let favorites = [favorite("notes"), favorite("claude")];
    let running = vec![app(2, "claude"), app(3, "chatgpt"), app(4, "notes")];
    let rows = order_menu(Some(app(1, "comet")), running, &favorites);
    assert_eq!(bundles(&rows), ["comet", "chatgpt", "notes", "claude"]);
    let starred: Vec<_> = rows.iter().filter(|row| row.favorite).collect();
    assert_eq!(starred.len(), favorites.len());
    // A running favorite shows its running instance (name and process), not the stored name.
    assert_eq!(rows[3].display_name, "CLAUDE");
    assert_eq!(rows[3].pid, Some(2));
}

#[test]
fn a_favorite_not_running_is_listed_without_a_process_and_its_stored_name() {
    let favorites = [favorite("notes")];
    let rows = order_menu(None, vec![app(3, "chatgpt")], &favorites);
    assert_eq!(bundles(&rows), ["chatgpt", "notes"]);
    assert_eq!(rows[1].pid, None);
    assert_eq!(rows[1].display_name, favorites[0].display_name);
    assert!(rows[1].favorite && !rows[1].frontmost);
}

#[test]
fn the_frontmost_favorite_is_listed_once_among_the_favorites() {
    let favorites = [favorite("comet")];
    let rows = order_menu(
        Some(app(1, "comet")),
        vec![app(1, "comet"), app(2, "chatgpt")],
        &favorites,
    );
    assert_eq!(bundles(&rows), ["chatgpt", "comet"]);
    assert!(rows[1].favorite && rows[1].frontmost);
}

#[test]
fn duplicate_running_identities_are_listed_once() {
    let rows = order_menu(
        None,
        vec![app(2, "chatgpt"), app(5, "chatgpt"), app(3, "notes")],
        &[favorite("notes")],
    );
    assert_eq!(names(&rows), ["CHATGPT", "NOTES"]);
    assert_eq!(rows[0].pid, Some(2));
}

#[test]
fn reorder_keeps_tokens_and_moves_a_starred_row_to_the_favorites() {
    let rows = order_menu(
        Some(app(1, "comet")),
        vec![app(2, "chatgpt"), app(3, "claude")],
        &[favorite("notes")],
    );
    let tokens: Vec<(u64, MenuTarget)> = (10..).zip(rows).collect();
    let favorites = [favorite("notes"), favorite("chatgpt")];
    let reordered = reorder(&tokens, &favorites);
    let order: Vec<_> = reordered
        .iter()
        .map(|(token, row)| (*token, row.favorite))
        .collect();
    assert_eq!(order, [(10, false), (12, false), (13, true), (11, true)]);
}

#[test]
fn reorder_drops_an_unstarred_favorite_that_is_not_running_and_restores_the_frontmost() {
    let rows = order_menu(
        Some(app(1, "comet")),
        vec![app(2, "chatgpt")],
        &[favorite("comet"), favorite("notes")],
    );
    let tokens: Vec<(u64, MenuTarget)> = (1..).zip(rows).collect();
    let reordered = reorder(&tokens, &[]);
    let rows: Vec<_> = reordered.into_iter().map(|(_, row)| row).collect();
    assert_eq!(bundles(&rows), ["comet", "chatgpt"]);
    assert!(rows.iter().all(|row| !row.favorite));
}
