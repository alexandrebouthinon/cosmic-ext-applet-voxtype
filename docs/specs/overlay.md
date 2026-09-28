# Overlay « orbe de particules »

## Besoin
L'utilisateur dicte avec Voxtype via un raccourci COSMIC et n'a aucun retour visuel. Une orbe de particules à l'écran lui montre que le micro l'écoute et réagit à sa voix.

## Parcours
1. L'utilisateur appuie sur le raccourci Voxtype : l'état passe à `recording`.
2. L'orbe apparaît en fondu (bas-droite de l'écran principal, par défaut) : environ 600 particules en orbite formant une boule (maquette 10b).
3. Pendant l'enregistrement, le rayon, la vitesse et le frémissement suivent le niveau du micro (`audio.sock`).
4. À la transcription (`transcribing`), l'orbe se contracte, tourne plus vite et passe à la couleur d'accent.
5. Retour à `idle` : fondu de sortie, la surface est masquée.

## Cas d'erreur et limites
- L'orbe n'existe que si l'applet est ajouté à un panneau ou au dock (COSMIC ne lance l'applet que dans ce cas) et que l'interrupteur « overlay » du popup est activé.
- `audio.sock` absent ou fermé : l'orbe s'affiche quand même, avec une animation au repos (niveau 0), et se reconnecte en tâche de fond.
- Daemon arrêté pendant l'affichage : l'orbe se masque.
- L'overlay ne prend jamais les clics ni le focus (surface « click-through »).
- Changement de thème COSMIC : les couleurs suivent sans redémarrage.

## Hors-périmètre
- Mode « animations réduites » (backlog).
- Support multi-écran (backlog) : une seule orbe quand l'applet tourne sur plusieurs panneaux (verrou d'instance, à cadrer en étudiant cosmic-applets), choix de l'écran. Test prévu avec un utilisateur qui a deux écrans.
- Démarrer ou arrêter l'enregistrement en cliquant sur l'orbe (backlog).

## Critères d'acceptation
- [ ] Applet retiré du panneau : aucune orbe, même pendant un enregistrement.
- [ ] L'orbe apparaît en moins de 150 ms après le passage à `recording`, et disparaît au retour à `idle`.
- [ ] Le rayon et la vitesse varient visiblement quand on parle, et restent au repos en silence.
- [ ] En `transcribing`, les couleurs passent à l'accent du thème.
- [ ] Un clic sur l'orbe atteint la fenêtre en dessous.
- [ ] Position (4 coins + bas-centre), taille (S/M/L) et activation se lisent dans la config de l'applet.
- [ ] Le processus overlay consomme moins de 5 % d'un cœur CPU pendant l'animation, et 0 % au repos (surface masquée, aucune boucle de rendu).

## Choix techniques
- Architecture : spike « l'applet peut-il ouvrir sa propre surface layer-shell ? » ; sinon, un 2e binaire lancé par l'applet (tranché par l'utilisateur).
- Rendu : `canvas` iced ; un shader wgpu seulement si le canvas rame (tranché).
- Couleurs dérivées du thème COSMIC : accent → étincelles et transcription ; violets et cyan de la palette → corps (tranché).
- Niveau audio : `$XDG_RUNTIME_DIR/voxtype/audio.sock`, trames de 16 octets à 100 Hz (seq u32, min f32, max f32, peak_dbfs f32).
- Écran : l'écran principal (tranché).

## Hypothèses
- Le compositeur choisit l'« écran principal » quand la surface layer-shell ne précise pas d'output : c'est le cas dans cosmic-osd (à valider au spike).
- Mapping des couleurs sur la palette du thème : 35 % violet (`accent_purple`), 25 % violet clair, 25 % cyan (`accent_blue`), 15 % accent (réglage fin possible plus tard).

## Tâches
| Lot | Tâche | Points | Fichiers | Fichier de référence |
|---|---|---|---|---|
| 0 | Client Voxtype : souscription `status --follow`, états idle/recording/transcribing/daemon absent/binaire absent | 3 | src/voxtype.rs | cosmic-applets/cosmic-applet-audio |
| 0 | Config : position, taille, overlay activé | 1 | src/config.rs | template |
| 1 | Spike : surface layer-shell depuis l'applet, input vide (click-through), output par défaut | 3 | src/overlay/ | cosmic-osd |
| 1 | Lecteur `audio.sock` : parsing des trames, reconnexion, lissage | 2 | src/overlay/audio.rs | voxtype src/osd/ipc.rs |
| 1 | Simulation de particules (port de 9a) + tests unitaires | 3 | src/overlay/particles.rs | maquette 10b |
| 1 | Rendu canvas + couleurs du thème | 3 | src/overlay/view.rs | libcosmic theme |
| 1 | Cycle de vie de la surface : afficher/masquer selon l'état, position, taille | 3 | src/overlay/mod.rs, src/app.rs | cosmic-osd |
