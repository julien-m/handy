# CLI destination hors dictée — native macOS (2026-10-28)

Build `handy-all-gates.app` (verrous ouverts, commit 8761d748, signée Apple Development), Handy lancé, aucune dictée en cours.

| Commande | Résultat | Exit |
|---|---|---|
| `destination status --json` | `{"pinned":null}` | 0 |
| `destination pin com.openai.chat --json` (ChatGPT non lancé) | `not_found` « No running application matches. » | 4 |
| `destination pin com.google.Chrome --json` | `pinned: {display_name: "Google Chrome", identity: bundle_id com.google.Chrome}` | 0 |
| `destination status --json` | même épingle | 0 |
| `destination unpin --json` | `{"pinned":null}` | 0 |
| `destination pin com.example.absent --json` | `not_found`, épingle inchangée (`status` → null) | 4 |

La révision reste 29 : l'épingle est un état de session, pas un réglage persistant. Reste : les mêmes commandes pendant une dictée, et l'effet sur la dictée en cours (avec Julien).
