# History limit — observation native macOS (2026-10-28)

Build : `handy-gates-closed.app` (hash dans `binary.sha256`), pilotée par Computer Use, vérifiée par `handy settings get history_limit --json`.

| Étape | UI | Store (révision) |
|---|---|---|
| Départ | 20 | 20 (16) |
| Saisie 1001 | « Saisissez un nombre entier de 0 à 1000. » (fr de `Enter a whole number from 0 to 1000.`) | 20 (16) : rien envoyé |
| 12 + Entrée | 12, message disparu | 12 (17) : un seul envoi |
| Blur (clic hors champ, changement de page, retour) | 12 | 12 (17) : valeur inchangée non renvoyée |
| Remise à 20 par la CLI | — | 20 (18) |
| 13 sans Entrée, puis activation du Finder | 13 | 20 (18) : rien enregistré |

Limites : Computer Use agit en arrière-plan ; la fenêtre Handy n'avait peut-être pas le focus clavier système avant l'activation du Finder, donc la perte de focus de la fenêtre n'est que partiellement prouvée. `hasFocus()` dans la webview n'a pas été lu (pas d'accès aux devtools de la build release).
