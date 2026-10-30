# Raccourcis par la CLI — observation native macOS (2026-10-28)

Build : `handy-gates-closed.app` (hash dans `binary.sha256`), backend `handy_keys`, Accessibilité accordée.

1. Pendant une dictée réelle de Julien (le log montre l'enregistrement/désenregistrement de `cancel` pendant la session), `handy settings shortcuts add transcribe ctrl+option+d --json` renvoie `ok:false`, `code: unavailable`, `reason: shortcut_session_busy` ; la révision reste 21.
2. Une fois Handy au repos, la même commande renvoie `ok:true`, révision 22, raccourcis `option+space`, `command_right`, `command_left+j`, `ctrl+option+d`. Le log montre `Registered handy-keys shortcut: transcribe -> … CTRL … OPT …` à 17:13:11 UTC, sans redémarrage.

Restent (avec Julien, clavier physique) : presser `ctrl+option+d` pour démarrer une dictée, `remove` puis vérifier qu'il est inerte, passage au backend Tauri ; volet « maintenir le push-to-talk pendant la commande » et bascule du post-processing.
