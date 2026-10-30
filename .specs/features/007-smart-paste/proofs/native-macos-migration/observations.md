# Migration AC-014 — observation native macOS (2026-10-28)

Build : `handy-all-gates.app` (`--features destination-return,smart-paste-dispatch`, commit `8761d748`, signée Apple Development ; hash dans `binary.sha256`).

- Avant (build verrous fermés, révision 27) : `paste_method: ctrl_v`, `reliable_paste: true`, `clipboard_handling: copy_to_clipboard`.
- Premier démarrage de la build verrous ouverts : révision 28, `paste_method: smart_paste`, `reliable_paste: false` (`output-after.json`), sans action de l'utilisateur.
- Réactiver `reliable_paste` par la CLI est refusé (`reliable-paste-refusal.json`).

Reste à observer : un choix volontaire de Cmd+V après migration doit survivre aux relances.
