# Revue indépendante — 003-recognition-vocabulary

- Relecteur : Claude (`claude-opus-5-5`), sans part à l'écriture de la spec ni du plan. Date : 2026-10-14. Branche `dev`, 008 mergée en `e9f96df`.
- Outil : `/opt/homebrew/bin/livespec` en mode dev (`/Users/julienm/projects/livespec`, commit `71f1e0d`). Budget `--review-max-chars 180000` partout.
- Aucune modification de `spec.md`, `plan.md` ni du code. Pas de cargo, pas de build.
- Lu : [spec](../../spec.md), [plan](../../plan.md), [constitution](../../../../constitution.md), [plan 008](../../../008-settings-cli/plan.md) (contexte de la revue), [implementation 008](../../../008-settings-cli/implementation.md), et le code 008 livré : `settings_service/{domains/mod.rs,descriptor_data.rs,errors.rs,protocol.rs}` et `README.md` (codes de sortie).

## Reçus

| Revue | Lots | Résultat | `context_hash` | Reçu (sha256) |
| --- | --- | --- | --- | --- |
| Spec | 1 | complete, **ready**, round 1 | `c5e679cf…7fa557966` | [`spec.json`](../../.reviews/spec.json) `462ea1c2…97bfe5` |
| Plan | 1 | complete, **ready**, round 1 (4 avertissements) | `6b9af60d…e419a219a8` | [`plan.json`](../../.reviews/plan.json) `89d07013…283f09` |

Sources : `spec.md` `e50af3db…ab125567f`, `plan.md` `7cd6167a…e82d44ca`.

## Progression

| Gate | Résultat | Preuve |
| --- | --- | --- |
| Clarify (`--progression plan`) | READY | [progression-plan.txt](progression-plan.txt) |
| Analyze (`--pre-impl`) | exit 0, aucun constat, couverture 15/15, sémantique `covered` | [analyze-pre-impl.json](analyze-pre-impl.json) |
| `--progression implement` | **READY** | [progression-implement.txt](progression-implement.txt) |
| `delivery lane` | **light** (criticité medium, rayon shared, aucune raison de voie complète) | [delivery-lane.json](delivery-lane.json) |

Les gates sémantiques autorisent l'implémentation. Restent hors de ces gates :

- la passe Penflow du programme avant l'étape 5 (UI) ;
- l'étape 1, qui doit prouver les budgets natifs par famille avant d'activer un adaptateur.

## Constats bloquants pour l'implémentation

Aucun au sens des gates. Deux points sont toutefois à trancher **avant l'étape 4 (CLI)**, car ils touchent le contrat livré de la 008 (voir plus bas) : les `issues` indexés et le refus `context_budget_unverified`.

## Écarts entre le plan et la 008 livrée

| Plan 003 | 008 livrée (`e9f96df`) | Effet |
| --- | --- | --- |
| `validation_failed` « with indexed issues » (ligne fautive) | `ErrorDetails` fermé : `issues[]` = `{field, reason}`, `field` limité aux clés de descripteurs, `reason` tiré d'un ensemble anglais fixe, `additionalProperties: false` | L'index par ligne n'entre pas dans le schéma. Il faut étendre la 008 (schéma, contrat, README) ou porter le détail dans les données d'aperçu. Rappel 008 : un dry-run invalide répond `validation_failed`/2, sans aperçu. |
| `unavailable`/5 avec raison `context_budget_unverified` | Exit 5 = « busy, starting, older, or the store cannot be used », relire avant de réessayer ; pas de champ `reason` | Code et forme inadaptés à un état durable ; état intermédiaire bloquant la release, mais à corriger |
| `vocabulary status` en direct par une requête bornée au worker | `Domain::plan(request, &AppSettings)` est pur (« no I/O and no live effect ») ; l'instance répond depuis l'instantané publié des réglages | Il faut étendre le cadre (port de lecture : capacités du worker, absent hors ligne). Le statut hors ligne (`unknown_until_model_loads`, depuis le modèle choisi) reste réalisable tel quel. |
| `vocabulary export FILE` (sortie atomique privée) | Le rendu 008 n'exporte que vers stdout (`View::Export`, `data.content` en JSON) | Chemin d'export fichier à ajouter |
| `LiveSettingsEffects` | N'existe pas : `effects.rs` derrière `LiveSettings` ; `custom_words` a un effet `None` (lu à l'usage), écritures UI via `tauri_adapter` | Renommer ; l'effet « lu à l'usage » convient à l'AC-006 |
| Refus du `set` générique de `custom_words` vers le domaine | Livré : `ValidationReason::VocabularyDomain`, propriétaire `vocabulary` | Conforme |
| Entrée fichier/stdin 1 MiB, dry-run, alias, `limit_exceeded`/2, exits 0–5 | Livrés ; domaine de test `words` déjà calqué sur cette liste | Conforme |

## Contradictions spec ↔ plan

- **Budget (AC-004, FR-003)** : la spec demande le « documented context budget » du modèle, fixé « from engine documentation ». Pour les vocabulaires génériques et les prompts de contexte, le plan impose un plafond applicatif de 223 tokens, « not a claimed engine limit ». On peut donc envoyer moins de termes que le budget documenté. La vérité reste sauve (termes non envoyés listés). L'écart est consigné comme ambiguïté non critique, à accepter ou corriger.
- **Statuts (AC-003)** : la spec impose exactement un statut parmi cinq. Le plan ajoute une erreur technique distincte, `context_budget_unverified`, pour une capacité positive au budget non prouvé, et bloque la release tant qu'elle peut apparaître. C'est acceptable comme garde-fou, à condition qu'elle ne sorte jamais en release.
- Le plan comble les manques de la spec relevés en revue :
  - doublons hérités et listes de plus de 500 termes conservés et signalés ;
  - statut CLI hors ligne `unknown_until_model_loads`.

## AC non testables ou fragiles

- SC-003 : un seul échantillon audio sur un seul modèle. Le résultat n'est pas déterministe au sens strict ; le plan le reconnaît (« No general accuracy improvement claim from one sample »).
- AC-003 et AC-004 pour Qwen, Fun-ASR et Granite : non testables tant que l'étape 1 n'a pas extrait les formules natives et prouvé les GGUF du catalogue. Le plan en fait un blocage d'activation, sans relabelliser en « not supported ».
- États visuels des traits : ils dépendent de la passe Penflow du programme.
- Le statut CLI en direct dépend de l'extension du cadre 008 décrite plus haut.

## `## Quality Engineering` ↔ `## QE Analysis`

- Dans la spec, les deux sections concordent : Medium ; rayon « Recognition quality for every dictation » / Shared ; risque statut trompeur et régression du prompt Whisper / UX puis Regression.
- **Le `## QE Analysis` du plan diverge** : criticité **High**, risque principal **Functional** (fausse efficacité), puis Data/Regression.
- La voie de livraison suit la spec et donne **light**, soit moins de rigueur que le plan ne s'en attribue. À aligner avant la livraison, dans un sens ou dans l'autre.

## Recommandations

1. Aligner la criticité : soit relever la spec en High via `spec-refine 003` (la voie deviendrait full), soit ramener le plan à Medium. Je recommande High, vu la préservation de données partagées et le risque de statut trompeur.
2. Avant l'étape 4, décider l'extension du contrat d'erreur 008 (index de ligne, raison typée) et le code de `context_budget_unverified` ; décrire l'extension de lecture du cadre de domaines (worker) et l'export fichier.
3. Dans l'étape 6, distinguer la preuve native macOS (gate d'Implemented) des observations Windows/Linux (*pending release evidence*), conformément à la constitution.
4. Accepter explicitement le plafond de 223 tokens, ou le remplacer par le budget documenté quand l'étape 1 l'aura extrait.
