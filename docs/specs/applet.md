# Applet du panneau

## Besoin
Voir l'état de Voxtype d'un coup d'œil, consulter ses infos et régler l'overlay sans passer par le terminal.

## Parcours
1. L'icône du panneau reflète l'état : repos, enregistrement, transcription, daemon arrêté (icône barrée).
2. Clic sur l'icône : un popup s'ouvre.
3. Le popup affiche le modèle, le device et le backend (`status --extended`).
4. Réglages de l'overlay dans le popup : activé/masqué, position (4 coins + bas-centre, bas-droite par défaut), taille (S/M/L).
5. Le choix du micro, dans le même popup, est décrit dans micro.md.

## Cas d'erreur et limites
- Daemon arrêté : icône barrée, infos affichées « indisponible », reconnexion automatique (backoff).
- Binaire `voxtype` absent : icône barrée, comme daemon arrêté (l'applet reste visible pour prouver qu'elle tourne).

## Hors-périmètre
- Démarrer ou arrêter l'enregistrement au clic (backlog).

## Critères d'acceptation
- [ ] L'icône change d'état en moins de 200 ms après un changement Voxtype.
- [ ] Arrêter `voxtype.service` donne l'icône barrée ; le relancer rétablit l'état normal sans action de l'utilisateur.
- [ ] Sans binaire `voxtype` dans le PATH, l'applet affiche l'icône barrée.
- [ ] Un changement de réglage d'overlay s'applique immédiatement et persiste après redémarrage de la session.

## Choix techniques
- Réglages dans cosmic-config (`src/config.rs`) ; l'overlay les lit en direct.

## Hypothèses
- Aucune.

## Tâches
| Lot | Tâche | Points | Fichiers | Fichier de référence |
|---|---|---|---|---|
| 1 | Icône selon l'état (y compris barrée), icône barrée si binaire absent | 2 | src/app.rs, resources/ | cosmic-applets/cosmic-applet-audio |
| 2 | Popup d'infos (modèle, device, backend) | 2 | src/app.rs | cosmic-applets/cosmic-applet-audio |
| 2 | Réglages de l'overlay dans le popup | 2 | src/app.rs, src/config.rs | cosmic-applets |
