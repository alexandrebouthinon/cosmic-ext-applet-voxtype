# Choix du micro

## Besoin
Choisir depuis le panneau le micro que Voxtype utilisera pour les prochaines dictées.

## Parcours
1. Dans le popup, une liste montre uniquement les périphériques d'entrée (micros), avec le micro actuel sélectionné (`voxtype config get audio.device`).
2. L'utilisateur choisit une entrée : `voxtype config set audio.device <nom>`, puis `systemctl --user restart voxtype`.
3. L'icône passe brièvement à « daemon arrêté », puis revient à l'état normal.

## Cas d'erreur et limites
- Enregistrement ou transcription en cours : la liste est désactivée.
- Échec du `config set` ou du redémarrage : message d'erreur dans le popup, la sélection revient à la valeur précédente.

## Hors-périmètre
- Tester le micro (vu-mètre) dans le popup.
- Capturer une sortie audio (moniteur) pour la transcrire (backlog, si l'usage apparaît).

## Critères d'acceptation
- [ ] Choisir un micro change `audio.device` et redémarre le service. La dictée suivante utilise ce micro.
- [ ] Impossible de changer de micro pendant un enregistrement.
- [ ] Une erreur est affichée et rien ne change si la commande échoue.

## Choix techniques
- Redémarrage automatique du service utilisateur (tranché par l'utilisateur).

## Choix fonctionnels
- Seulement les entrées : pas de sorties ni de moniteurs dans la liste (tranché par l'utilisateur).

## Hypothèses
- `voxtype info devices --json` mélange entrées et sorties ALSA (`front:`, `surround…`). La méthode de filtrage (liste Voxtype filtrée ou sources PipeWire hors `.monitor` acceptées par `audio.device`) se tranche en début de tâche, avec un test réel (non bloquant).

## Tâches
| Lot | Tâche | Points | Fichiers | Fichier de référence |
|---|---|---|---|---|
| 2 | Liste des devices, sélection, config set + restart, erreurs | 3 | src/voxtype.rs, src/app.rs | cosmic-applets/cosmic-applet-audio |
