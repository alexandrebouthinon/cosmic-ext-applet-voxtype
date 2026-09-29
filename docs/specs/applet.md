# Applet du panneau

## Besoin
Voir l'état de Voxtype d'un coup d'œil, consulter ses infos et régler l'overlay sans passer par le terminal.

## Parcours
1. L'icône du panneau reflète l'état : repos, enregistrement, transcription, daemon arrêté (icône barrée).
2. Clic sur l'icône : un popup s'ouvre (disposition B de la maquette : https://claude.ai/artifact/Twxfwdprz56zxNrgHM3Rqp).
3. En-tête fixe : glyphe d'état, « Voxtype », texte d'état, toggler du daemon (daemon.md).
4. Onglets (`segmented_button`) :
   - Statut : modèle et backend (`voxtype status --format json --extended`) ;
   - Micro : liste des entrées (micro.md) ;
   - Orbe : toggle affichée/masquée, position en dropdown (4 coins + bas-centre, bas-droite par défaut), taille S/M/L en segmented.

## Cas d'erreur et limites
- Daemon arrêté : icône barrée, onglet Statut « indisponible », reconnexion automatique (backoff).
- Binaire `voxtype` absent : icône barrée, comme daemon arrêté (l'applet reste visible pour prouver qu'elle tourne).

## Hors-périmètre
- Démarrer ou arrêter l'enregistrement au clic (backlog).
- Champ `device` de `status --extended` : c'est le micro, affiché dans l'onglet Micro seulement.

## Critères d'acceptation
- [ ] L'icône change d'état en moins de 200 ms après un changement Voxtype.
- [ ] Arrêter `voxtype.service` donne l'icône barrée ; le relancer rétablit l'état normal sans action de l'utilisateur.
- [ ] Sans binaire `voxtype` dans le PATH, l'applet affiche l'icône barrée.
- [ ] Le popup affiche le modèle et le backend réels ; « indisponible » quand le daemon est arrêté.
- [ ] Position et taille de l'orbe se règlent dans l'onglet Orbe.
- [ ] Un changement de réglage d'overlay s'applique immédiatement et persiste après redémarrage de la session.

## Choix techniques
- Disposition : en-tête + onglets (B) ; position en dropdown (tranché par l'utilisateur).
- Réglages dans cosmic-config (`src/config.rs`) ; l'overlay les lit en direct.

## Hypothèses
- Aucune.

## Tâches
| Lot | Tâche | Points | Fichiers | Fichier de référence |
|---|---|---|---|---|
| ✅ | Icône selon l'état (y compris barrée), icône barrée si binaire absent (sprint orbe) | 2 | src/app.rs, resources/ | cosmic-applets/cosmic-applet-audio |
| ✅ | Squelette du popup : en-tête, onglets, messages ; wrappers CLI (`status --extended`, `systemctl --user`, devices) dans src/voxtype.rs | 3 | src/app.rs, src/voxtype.rs | cosmic-applets/cosmic-applet-audio |
| ✅ | Onglet Statut (modèle, backend, « indisponible ») | 2 | src/app.rs | cosmic-applets/cosmic-applet-audio |
| ✅ | Onglet Orbe (toggle, dropdown position, segmented taille) | 2 | src/app.rs, src/config.rs | cosmic-applets |
