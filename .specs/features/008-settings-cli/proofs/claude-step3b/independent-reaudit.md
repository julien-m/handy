# Re-audit indépendant — étape 3 de 008-settings-cli (dépôt atomique et révision)

- Périmètre : `git diff f3e268b..58d1fdc` (correctifs du commit `42b4d41`), avec une vue d'ensemble de l'étape 3 (`git diff e3bc314..58d1fdc`). Branche `feature/008-settings-cli`.
- Sources lues : AGENTS.md, constitution, `.conventions/index.md` → `general.md` (§6, §24, §25) et `rust.md`, spec (AC-003/004/005), plan § Paths/storage/revision, § Step 3 et § Step 4, `implementation.md` (dispositions de l'audit indépendant), premier audit `proofs/claude-step3/independent-audit.md`, reproductions `scratchpad/repro/tests/repro.rs`.
- Mode : lecture seule. Ce rapport est le seul fichier écrit dans le dépôt. `git status --short -- src-tauri` est vide. Les nouvelles reproductions sont dans le crate jetable hors dépôt `scratchpad/repro/tests/reaudit.rs` (journal : `scratchpad/repro/reaudit.log`).

## Tests relancés

| Commande | Résultat |
| --- | --- |
| `cargo test --manifest-path src-tauri/Cargo.toml --test settings_transactions` | exit 0 : 36 réussis, 0 échec, 1 ignoré (point d'entrée du sous-processus `driver::repository_subprocess_fixture`) |
| `cargo test --manifest-path src-tauri/Cargo.toml --lib settings_service` | exit 0 : 5 réussis, 0 échec (`atomic::tests::*` ×2, `raw_merge::tests::*` ×2, `locator::tests::app_identifier_matches_the_tauri_bundle_identifier`) |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --lib --test settings_transactions -- -D warnings` | exit 0 |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | exit 0 |
| Anciennes reproductions `repro.rs` (4) | exit 0. Les trois cas du blocker sont corrigés (voir plus bas). `{}` se lit maintenant comme une nouvelle installation. |
| Nouvelles reproductions `reaudit.rs` (10) | exit 0. Les invariants sont vérifiés, sauf deux observations reprises en constats (R2, R3). |

## Vérification des constats du premier audit

| # | Sévérité | État | Preuve |
| --- | --- | --- | --- |
| 1 | blocker | Corrigé | `apply_mutation` persiste `normalized_payload(stocké, courant)` plus le delta, avec une seule révision. `repro.rs` : `gpu-handle-1` relu tel quel (révision 1), `overlay_style` reste `none`, `selected_language` reste `zh`. Balayage `legacy_sweep_returned_equals_reread` : 7 formes de store × 5 écritures. Le snapshot renvoyé est toujours égal à la relecture, et aucun autre champ visible ne change. |
| 2 | major | Corrigé | `commit_owned(&StartupLease, …)` prend le verrou de transaction, relit, applique le CAS et revalide le bail détenu, sans sonde passive. Test du dépôt et reproduction : `initialize` donne la révision 1, `commit_owned` la révision 2. Un écrivain hors ligne reste refusé (`unavailable`), un bail d'un autre store est refusé (`unavailable`), un CAS obsolète du propriétaire est refusé. |
| 3 | minor | Corrigé | `{}` donne `fresh_install()` (`whats_new_last_seen_version` = `0.9.8`). `{"settings":{}}` reste une mise à jour (`""`), et une écriture conserve chacune de ces deux valeurs. |
| 4 | minor | Corrigé | Sur un JSON cassé, `[1,2]`, `"x"`, `{"settings":null}` et `{"settings":[1]}` : lecture avec `warning`, `initialize` garde le bail, les écritures hors ligne et propriétaire renvoient `persistence_failed`, les octets restent identiques et aucun fichier `.tmp` n'est créé. |
| 5 | minor | Corrigé | `settle_absent_store` décide refus et no-op avant toute création. Le cas d'un store existant est documenté sur `commit_with_probe`. Voir le nit R3 sur le bail. |
| 6 | minor | Corrigé sur Unix ; régression sur Windows | L'autorisation complète tourne avant la création du fichier temporaire et juste avant le remplacement. Sur Unix, le nettoyage vérifie l'identité du fichier avant de le supprimer, et les tests vérifient qu'il ne reste aucun `.tmp`. Sur Windows, le nettoyage peut supprimer le store validé (constat R1). |
| 7 | minor | Corrigé | Un lien du même utilisateur vers un dossier du même utilisateur est suivi. Un lien vers un dossier d'un autre utilisateur ou vers le store est refusé. Voir le nit R4 sur les chaînes de liens. |
| 8 | minor | Corrigé | `nearest_existing_ancestor` et `validate_canonical_ancestor` sont extraits (tous deux ≤ 30 lignes). |
| 9 | minor | Non corrigé (report justifié, inchangé) | Code FFI Windows impossible à exécuter sur l'hôte macOS. À suivre avec les preuves Windows avant la publication. |
| 10 | minor | Corrigé | `apply_changed_fields` supprime les clés perdues et descend dans les objets. Couvert par des tests unitaires. Les champs inconnus imbriqués survivent (reproduction `unknown_fields_survive_normalized_write`). |
| 11 | minor | Corrigé | L'écrivain 1 est mis en pause à `AfterLoad`. L'écrivain 2 ne donne aucun résultat pendant 500 ms. Les révisions valent exactement 1 puis 2. |
| 12 | nit | Corrigé | Le test est renommé, et la parité avec Tauri via `dirs 6.0.0` est documentée. |
| 13 | nit | Corrigé | Un test déplace le store et les deux verrous dans un parent substitué. Seule la comparaison du parent refuse, et aucun `.tmp` ne reste. |
| 14 | nit | Corrigé | `resolve_data_dir` lit les octets et applique la même règle que `init` (marqueur existant, magique ou à côté de `Data`). Vérifié aussi par lecture pour un lien pendant et un marqueur qui est un dossier. |
| 15 | nit | Corrigé | `STORE_FILENAME` réutilise `SETTINGS_STORE_PATH`, et un test compare l'identifiant à `tauri.conf.json`. |
| 16 | nit | Corrigé | Une ligne de doc par champ de `SettingsRead` et par variante de `PersistStage`. |
| 17 | nit | Corrigé | Commentaire en ligne sur le repli JSON. |

## Contrôle de non-régression

- Pas de double incrément : une écriture sur un store hérité donne la révision 1, puis `initialize` laisse la révision à 1 sans réécrire le disque, puis `commit_owned` donne la révision 2. Dans l'autre ordre, `initialize` donne 1 puis `commit_owned` donne 2 (reproductions `no_double_increment_commit_then_initialize`, `initialize_then_owner_commit_on_legacy`).
- Pas de perte de champs inconnus : sont conservés au premier niveau, dans le payload, dans une entrée de `bindings`, dans un élément de prompt (id unique) et dans un fournisseur de clé API présent seulement dans le JSON brut. Un secret brut invalide (`{"openai":5}`) est conservé tel quel après une écriture.
- Nettoyage du fichier temporaire sur Unix : sûr (comparaison dev/ino avec le handle détenu, et un substitut est conservé, voir `faults::replaced_temporary_path…`). Sur Windows : non sûr après le renommage (R1).
- Liens symboliques : conformes au plan pour les liens directs. Chaînes de liens : voir R4.
- `{}` : traité comme une nouvelle installation. Un payload existant, même vide, reste traité comme une mise à jour réelle.
- JSON corrompu : la lecture de secours fonctionne et aucune écriture n'est autorisée (constat 4 ci-dessus).
- Aucun `unwrap`/`expect`/`panic!`/log ajouté dans le code de production de l'étape. Les affectations par index `serde_json` (`document.raw["settings"]`) ne portent que sur des documents objets (`writable` est faux sinon). Les messages d'erreur ne contiennent aucun secret.
- Taille des fonctions modifiées : ≤ 30 lignes (`persist` 25, `merge_change` 25, `commit` 23). Le code public ajouté est documenté (`commit_owned`, `FileLock::path`, `discard_owned`, `has_portable_magic`).

## Constats

| # | `fichier:ligne` | Sévérité | Problème | Correctif proposé |
| --- | --- | --- | --- | --- |
| R1 | `src-tauri/src/settings_service/atomic.rs:72` (`discard_temp` l. 115) ; `windows_store/replace.rs:32` | major | Régression introduite par `42b4d41` sur Windows. `replace_owned` renvoie `ReplaceError::After` quand le renommage a déjà remplacé le store mais que le `FlushFileBuffers` qui suit échoue. `persist` traite cette erreur comme les autres et appelle `discard_temp(&temporary, &file)`. La branche Windows appelle alors `discard_owned(held)`, qui marque pour suppression (`FileDispositionInfo`, handle ouvert avec `DELETE`) le handle qui désigne maintenant `settings_store.json`. Le store validé (secrets compris) est supprimé à la fermeture du handle. Le résultat annoncé est « unknown outcome », mais la relecture conseillée renvoie les valeurs par défaut et la révision 0, ce qui casse la continuité de révision (plan : « Never silently reset the counter »). Sur Unix, le cas n'existe pas : après le renommage, le chemin temporaire n'existe plus. Ce constat vient de la lecture du code, car Windows ne peut pas s'exécuter sur l'hôte. | Ne supprimer le fichier temporaire que pour une erreur antérieure au remplacement. Par exemple : faire remonter la phase (`Before`/`After`) de `ParentDirectory::replace` jusqu'à `persist`, appeler `discard_temp` seulement sur `stage_generation`, la seconde autorisation ou `Before`, et ne jamais l'appeler sur `After`. Ajouter un test d'injection de faute après le remplacement qui vérifie que le store survit, à exécuter sur l'agent Windows. |
| R2 | `src-tauri/src/settings_service/transaction.rs:304` (`apply_mutation`) ; `settings.rs:1214` ; `descriptor_data.rs:75` | minor | Une mutation acceptée peut produire un état que la normalisation de lecture réécrit. Reproduit avec `transcribe_accelerator=gpu` alors que `transcribe_gpu_device` vaut `null` : le commit réussit (révision 5 → 6), renvoie `"gpu"` et écrit `"gpu"` sur le disque, puis la relecture suivante donne `"auto"`, car la migration `Gpu && device none → Auto` tourne à chaque lecture. L'écriture est perdue en silence et le snapshot renvoyé diffère de la relecture. Ce n'est pas une régression : le défaut existait dès `1ae5d84` et le premier audit ne l'a pas relevé. L'interface l'évite en écrivant d'abord le device (commentaire dans `AccelerationSelector.tsx`), mais un `set` générique l'atteint. | Dans `apply_mutation`, renormaliser le candidat (même chemin que `normalization::normalize`). Si la valeur mutée change, refuser avec `validation_failed` (règle croisée « `gpu` exige `transcribe_gpu_device` »), ou persister et renvoyer la valeur stabilisée. Ajouter un test de régression. Signaler aussi la règle croisée à la parité de validation (AC-003). |
| R3 | `src-tauri/src/settings_service/transaction.rs:152` (`settle_absent_store`) | nit | Sur un store absent, le chemin rapide ne valide pas le bail. `commit_owned` avec le bail d'un autre store renvoie `Ok` pour un no-op, alors qu'une mutation réelle renvoie `unavailable`. Un écrivain hors ligne pendant qu'une instance détient le bail (store supprimé pendant l'exécution) renvoie `Ok` pour un no-op au lieu de `unavailable`. Reproduit (`foreign_lease_noop_on_absent_store`, `offline_noop_absent_store_with_running_instance`). Rien n'est écrit, mais le code d'erreur varie selon que le store existe ou non. | Pour `Authority::Owner`, comparer `lease.lock.path()` au chemin du bail attendu avant tout no-op. Pour un écrivain hors ligne, si le parent existe, tester le bail d'instance en mode non bloquant avant de renvoyer un no-op. Ajouter un test. |
| R4 | `src-tauri/src/settings_service/locator.rs:129` (`is_permitted_alias`) | nit | `std::fs::metadata(link)` suit toute la chaîne, et `reject_aliases` n'examine que les composants du chemin d'origine. Un lien de l'utilisateur courant qui passe par un lien intermédiaire appartenant à un autre utilisateur (par exemple `~/a` → `/tmp/b`, lien de l'autre utilisateur, → `~/real`) est donc accepté sans que le propriétaire du lien intermédiaire soit vérifié. Le plan ne vise pourtant que les « aliases that cross user boundaries ». L'impact reste faible : la cible canonique doit appartenir à l'utilisateur courant, et les opérations suivantes portent sur le chemin canonique. Constat issu de la lecture du code, non reproduit faute d'un second compte. | Suivre un seul saut (`read_link`) et réappliquer `reject_aliases` à la cible jointe, avec une profondeur bornée. Sinon, documenter que seul le premier lien est contrôlé. |

Note pour l'étape 4, hors constat : `initialize` exécute la sonde passive des anciennes instances avant chaque remplacement. Elle doit donc tourner avant que l'instance n'enregistre ses propres ressources single-instance, sinon elle se verrait comme propriétaire. Le commentaire de `Fence::Owner` reconnaît ce point pour `commit_owned` seulement.

## Bilan

- blocker : 0
- major : 1 (R1)
- minor : 1 (R2)
- nit : 2 (R3, R4)
- Premier audit : 15 constats corrigés sur 17. Constat 9 : report justifié et inchangé. Constat 6 : corrigé sur Unix, régression Windows (R1).

## Verdict

**FAIL** : le major R1 (nettoyage Windows qui peut supprimer le store validé après le renommage) bloque. Le blocker et le major du premier audit sont corrigés.
