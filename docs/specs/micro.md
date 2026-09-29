# Choix du micro

## Besoin
Choisir depuis le panneau le micro que Voxtype utilisera pour les prochaines dictées.

## Parcours
1. Dans l'onglet Micro du popup, une liste montre uniquement les périphériques d'entrée (micros), avec le micro actuel sélectionné (`voxtype config get audio.device`).
2. L'utilisateur choisit une entrée : `voxtype config set audio.device <nom>`, puis `systemctl --user restart voxtype`.
3. L'icône passe brièvement à « daemon arrêté », puis revient à l'état normal.

## Cas d'erreur et limites
- Enregistrement ou transcription en cours : la liste est désactivée.
- Daemon arrêté par l'utilisateur : `config set` seul, sans redémarrage ; le micro s'applique au prochain démarrage (message dans l'onglet).
- Échec du `config set` ou du redémarrage : message d'erreur dans le popup, la sélection revient à la valeur précédente.

## Hors-périmètre
- Tester le micro (vu-mètre) dans le popup.
- Capturer une sortie audio (moniteur) pour la transcrire (backlog, si l'usage apparaît).

## Critères d'acceptation
- [ ] Choisir un micro change `audio.device` et redémarre le service. La dictée suivante utilise ce micro.
- [ ] Daemon arrêté : le choix est enregistré sans relancer le daemon.
- [ ] Impossible de changer de micro pendant un enregistrement.
- [ ] Une erreur est affichée et rien ne change si la commande échoue.

## Choix techniques
- Redémarrage automatique du service utilisateur (tranché par l'utilisateur).

## Choix fonctionnels
- Seulement les entrées : pas de sorties ni de moniteurs dans la liste (tranché par l'utilisateur).
- Filtrage : liste de `voxtype info devices --json` sans les sorties ALSA (`front:`, `surround*`, `hdmi`, `iec958`…) ; les noms de sources PipeWire sont refusés par Voxtype (`Audio device not found`, testé) (tranché par l'utilisateur).

## Hypothèses
- Aucune.

## Tâches
| Lot | Tâche | Points | Fichiers | Fichier de référence |
|---|---|---|---|---|
| 1 | Liste des devices, sélection, config set + restart, erreurs | 3 | src/voxtype.rs, src/app.rs | cosmic-applets/cosmic-applet-audio |
