# Choix du micro

## Besoin
Choisir depuis le panneau le micro que Voxtype utilisera pour les prochaines dictées.

## Parcours
1. Dans l'onglet Micro du popup, une liste montre les sources PipeWire (`pw-dump`, `media.class` `Audio/Source*`, virtuelles comme Easy Effects comprises, sans moniteurs), sous leur nom lisible (`node.description`), avec la source par défaut sélectionnée.
2. L'utilisateur choisit une source : `pactl set-default-source <nom>`. Une mention indique que le micro change pour toutes les applis.
3. Si Voxtype n'est pas sur `audio.device = default` : `voxtype config set audio.device default`, puis `systemctl --user restart voxtype` ; l'icône passe brièvement à « daemon arrêté ».

## Cas d'erreur et limites
- Enregistrement ou transcription en cours : la liste est désactivée.
- Daemon arrêté par l'utilisateur : `config set` seul, sans redémarrage ; le passage à `default` s'applique au prochain démarrage (message dans l'onglet).
- Aucune source : message « aucun micro » dans l'onglet.
- Échec de `pactl` : message d'erreur, la sélection ne change pas. Échec du `config set` ou du redémarrage : message d'erreur, la source par défaut reste changée.

## Hors-périmètre
- Tester le micro (vu-mètre) dans le popup.
- Capturer une sortie audio (moniteur) pour la transcrire (backlog, si l'usage apparaît).

## Critères d'acceptation
- [ ] La liste montre des noms lisibles (ex. Blue Microphones, Easy Effects Source), sans sorties ni moniteurs.
- [ ] Choisir un micro change la source PipeWire par défaut ; la dictée suivante utilise ce micro.
- [ ] Voxtype hors `default` : il y est remis et redémarré ; daemon arrêté, sans le relancer.
- [ ] Impossible de changer de micro pendant un enregistrement.
- [ ] Une erreur est affichée et rien ne change si la commande échoue.

## Choix techniques
- Redémarrage automatique du service utilisateur (tranché par l'utilisateur).

## Choix fonctionnels
- Seulement les entrées : pas de sorties ni de moniteurs dans la liste (tranché par l'utilisateur).
- Source des micros : PipeWire, pas `voxtype info devices` (PCM ALSA bruts, illisibles). Voxtype refuse les noms de sources PipeWire (`Audio device not found`, testé) : il reste sur `default` et l'applet change la source par défaut du système (tranché par l'utilisateur en recette).
- `pw-dump` plutôt que `pactl --format=json` : ce dernier produit un JSON invalide avec des descriptions accentuées (pactl 17).

## Hypothèses
- Aucune.

## Tâches
| Lot | Tâche | Points | Fichiers | Fichier de référence |
|---|---|---|---|---|
| ✅ | Liste des devices, sélection, config set + restart, erreurs | 3 | src/voxtype.rs, src/app.rs | cosmic-applets/cosmic-applet-audio |
