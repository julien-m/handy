# Vocabulaire — observation native macOS (2026-10-28)

Build : `handy-gates-closed.app` (hash dans `binary.sha256`), modèle chargé Parakeet TDT 0.6B v3 (sans support du vocabulaire), pilotée par Computer Use.

- **Statut avec un modèle sans support** : l'UI affiche « Parakeet TDT 0.6B v3 ne peut pas utiliser le vocabulaire pendant la reconnaissance… Choisissez un modèle Whisper… ». `vocabulary status --json` renvoie `status: not_supported`, `terms_sent: []` et chaque terme dans `terms_not_sent` avec `reason: no_context` (`status-parakeet.json`). Les deux concordent.
- **Collage d'une liste** : liste `OpenClaw / LiveSpec / claude code / Penflow / (vide) / 120 × x / Tauri` → aperçu « À ajouter : 4 · 1 doublon · Erreurs : 2 », doublon « claude code — identique à Claude Code », erreurs « ligne vide » et « 100 caractères au maximum », bouton « Ajouter les termes (4) » désactivé, message « Votre vocabulaire n'a pas changé ». Liste corrigée → bouton actif, 4 termes ajoutés, compteur « 5 sur 500 ».
- **Suppression** : supprimer le dernier terme (Tauri) → compteur « 4 sur 500 », focus dans le champ « Ajouter un terme » (curseur visible sur `vocabulary-unsupported.png`), store à 4 termes (`custom_words`, révision 21).
- Non couvert : statut avec Whisper chargé et sans modèle chargé, navigation Tab complète, Apply en échec puis Retry, effet réel sur la dictée.
- Outil : Computer Use ferme l'app cible sur Cmd+A (reproduit aussi sur Calculette) ; ce n'est pas un défaut de Handy. Les captures `screencapture` d'une fenêtre en arrière-plan peuvent être figées : la barre latérale y est périmée.
