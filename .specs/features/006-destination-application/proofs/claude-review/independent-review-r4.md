# Revue indépendante, round 4 — 006-destination-application

- Relecteur : Claude (`claude-opus-5-5`). Je n'ai écrit ni la spec, ni le plan, ni le code. Date : 2026-10-14, branche `feature/006-destination-application`, HEAD relu `88185b1`.
- Outil : `/opt/homebrew/bin/livespec` (`livespec` `71f1e0d`), avec `--model claude-opus-5-5 --review-max-chars 180000` sur chaque commande.
- Je n'ai modifié ni `spec.md`, ni `plan.md`, ni les contrats, ni le code. Aucun cargo.
- Point de départ : `--progression implement` renvoyait `BLOCKED: clarify_not_ready:stale_or_incomplete_spec_review`, parce que AC-005 et le plan avaient changé (`306c368`).
- Sources relues : `spec.md` `0a548015…cf5fb936`, `plan.md` `26ac1224…603648`, `contracts/destination-contract.md` `c3bc0cf0…759f1`.

## Résultats

| Étape | Résultat | Preuve |
| --- | --- | --- |
| Préparation de la revue de spec | 1 lot (45 022 caractères), aucune erreur, `context_hash` `8d6d7114…5f64e` | [prepared-spec-r4.json](../../.reviews/prepared-spec-r4.json) |
| Revue de spec | **complete, ready** ; 17/17 `covered` ; 4 ambiguïtés non critiques (dont 1 résolue par la décision du 2026-10-14) ; 0 bloquant ; 2 avertissements, 3 infos ; 1 portée métier approuvée | [spec.json](../../.reviews/spec.json), sha256 `b0242cf9…69ad7` |
| Clarify (`--progression plan`) | **READY** | [progression-plan-r4.txt](progression-plan-r4.txt) |
| Préparation de la revue de plan | 1 lot (162 405 caractères, sous le budget de 180 000), aucune erreur, `context_hash` `9058849c…ab6388` | [prepared-plan-r4.json](../../.reviews/prepared-plan-r4.json) |
| Revue de plan | **complete, ready** ; 18/18 `covered` ; 0 ambiguïté ; 0 bloquant ; 4 avertissements, 2 infos ; 3 portées techniques justifiées | [plan.json](../../.reviews/plan.json), sha256 `0da91f14…4cce7` |
| Tours | spec : round 1 sur ce nouvel état ; plan : round 1 sur ce nouvel état ; `round_cap_reached: false` pour les deux | [spec.rounds.json](../../.reviews/spec.rounds.json), [plan.rounds.json](../../.reviews/plan.rounds.json) |
| Analyze (`--pre-impl`) | exit 0 ; 0 constat ; couverture structurelle 17/17 ; statut sémantique `covered` | [analyze-pre-impl-r4.json](analyze-pre-impl-r4.json) |
| `--progression implement` | **READY** | [progression-implement-r4.txt](progression-implement-r4.txt) |
| `delivery lane` | **full** (`criticality_high`, rayon `shared`) | [delivery-lane-r4.json](delivery-lane-r4.json) |

`context_hash` complets : spec `8d6d711437b539df22a549f53dda9eb46bb5a488bab08df364670326def5f64e`, plan `9058849cdd511a72f22f1eb4333efae20be6cb16d6ae822ca2dc19c842ab6388`.

Aucun lien Markdown normatif n'a fait dépasser le budget : la revue de plan tient en un lot.

Incident de procédure : la première ingestion de la revue de spec a été refusée (`invalid_citation` : AC-007 citait la section Clarifications, hors de ses sections). J'ai remplacé cette citation par une citation de la section d'AC-007, puis réingéré. Le refus n'a pas compté comme tour. Le jugement est resté le même.

## Revue de spec : AC-005 révisé

- AC-005 dit maintenant que la transcription reste dans l'historique dès que l'enregistrement en historique réussit, sous forme d'entrée texte seul si le WAV n'a pas pu être enregistré. La Clarification datée du 2026-10-14 donne la raison. Le critère est testable et couvert.
- **Ambiguïté résolue (non critique)** : AC-005 place la règle du texte seul dans le critère des échecs de destination, alors que la décision l'étend à toute dictée dont le WAV échoue. La décision fait foi : la règle vaut pour toutes les dictées.
- **Portée métier approuvée** : l'entrée texte seul pour toute dictée dont le WAV échoue change l'enregistrement de l'historique en général, et pas seulement le flux de destination. C'est une décision explicite et datée de l'utilisateur.
- Avertissements reportés au plan :
  1. la règle est transverse et ne figure que dans les Clarifications. Le plan doit la porter comme une étape partagée. **Le plan le fait** (« History rule » et « Shared history change »).
  2. si l'enregistrement en historique échoue lui-même, seul le passage à la 010 conserve le texte, et la 010 est livrée après la 006. **Le plan le traite** : emplacement volatil, et AC-005 en ⚠️ Partial pour le cas « overlay none » tant que la 010 n'est pas livrée.
- Trois ambiguïtés non critiques restent ouvertes. Le plan les tranche :
  - lecture du réglage figée au démarrage de la dictée ;
  - tableau des résultats transmis à 009 et 010 ;
  - `RecoveryPort`, avec l'accusé de réception de la 010.

## Revue de plan : nouvel état

- La règle d'historique est appliquée de façon cohérente dans le plan et dans le contrat :
  - `history_status` vient du seul résultat de `save_entry` ;
  - la condition sur `wav_saved` disparaît ;
  - `file_name` devient nullable par reconstruction transactionnelle de la table ;
  - les fonctions qui supposent un fichier audio sont protégées (lecture, nouvelle transcription refusée avec sa raison, export, nettoyage, interface) ;
  - la matrice de l'étape 6 couvre : WAV enregistré, échec du WAV (ligne texte seul `saved`), échec de `save_entry` (`failed`), annulation (`not_saved`).
- Le travail sur l'historique est partagé avec l'étape 7 de la 005 : la première des deux à être implémentée le construit, et la 005 reprend la même règle.

### Avertissements (`carry_over`, non bloquants)

1. **Ordre de l'étape 6.** L'étape 6 exige que les surfaces UI de l'étape 5 soient prêtes, donc la certification Penflow du programme. Pourtant, sa partie historique ne dépend d'aucune UI. L'historique de la branche contient déjà cette partie (`8f374d4`), ainsi que la commande de réglage UI de l'étape 3 (`3fd745d`). Ces deux commits sont antérieurs au READY d'`implement` de ce round. Je propose de scinder l'étape 6 en deux : une sous-étape historique et une sous-étape raisons et consommateurs.
2. **Retour à une version antérieure de Handy.** Le plan ne dit pas comment un binaire plus ancien se comporte face à la base migrée (version de migration plus récente, lignes avec `file_name` NULL). Le Constitution Check annonce, pour un ancien binaire Handy, une vérification « exacte » nommée à l'étape 7, mais l'étape 7 ne nomme que Windows et Linux.
3. **Alignement de 007/009/010.** Le plan reconnaît que ces plans décrivent encore un historique conditionné par le WAV, et laisse leur réalignement à leurs propriétaires, sans contrôle.
4. **Repris du round 3.** L'ordre exact délai → garde → `DestinationContext` → `before_insertion` → `set_disposition(Insert)` → envoi n'est pas fixé à travers la frontière `clipboard.rs` / orchestrateur.

### Infos

- L'étape 0 (opérations live, `details.items`) ne sert aucune exigence de la 006. Elle est tracée vers FR-006 seule, ce qui est trompeur. Je l'accepte comme portée technique décidée par l'orchestrateur (D2–D4).
- La liste des champs de `DestinationSession` et le diagramme ER omettent `enter_claimed`. L'entité `INSERTION_OUTCOME` omet `history_entry_id`.

### Portées techniques acceptées

- Extension du framework de réglages (étape 0), justifiée par les décisions D2–D4 pour 001, 003, 005 et 009.
- Migration et protections pour les lignes sans audio : c'est une nécessité technique de l'AC-005 révisé.
- Emplacement volatil de passage à la 010, sans durabilité revendiquée.

## Constats bloquants restants

Aucun.
