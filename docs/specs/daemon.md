# Arrêt et relance du daemon

## Besoin
Libérer le GPU (pour un jeu, par exemple) en arrêtant Voxtype depuis le panneau, puis le relancer quand on veut dicter.

## Parcours
1. Le toggler de l'en-tête du popup reflète `systemctl --user is-active voxtype`.
2. Le couper : `systemctl --user stop voxtype`, effet immédiat, même pendant une dictée. Message « GPU libéré, Voxtype redémarrera à la prochaine connexion ».
3. L'activer : `systemctl --user start voxtype` ; l'applet se reconnecte au flux d'état.

## Cas d'erreur et limites
- Arrêté par l'utilisateur (`inactive`) et en échec (`failed`) : même icône barrée ; l'en-tête dit « Arrêté » ou « En échec » avec l'indice `journalctl --user -u voxtype`.
- Unité absente : toggler désactivé, message « Service introuvable : lancez `voxtype setup systemd` ».
- Échec du stop ou du start : erreur inline sous l'en-tête, le toggler revient à l'état réel.
- Arrêté hors applet (terminal) : l'état suit, sans action.

## Hors-périmètre
- Désactiver le démarrage à la connexion (`systemctl disable`).
- Relance automatique à la fermeture d'un jeu.

## Critères d'acceptation
- [ ] Couper le toggler arrête le service en moins d'une seconde ; `nvidia-smi` (ou équivalent) ne montre plus Voxtype.
- [ ] L'activer relance le service et l'icône revient à l'état normal sans autre action.
- [ ] `inactive` et `failed` s'affichent différemment dans le popup.
- [ ] Sans unité `voxtype.service`, le toggler est désactivé et le message donne la commande.
- [ ] Un échec de commande s'affiche et le toggler reflète l'état réel.

## Choix techniques
- `systemctl --user` via la CLI, pas de D-Bus (tranché par l'utilisateur).
- Arrêt simple, même en pleine dictée (tranché par l'utilisateur).

## Hypothèses
- Aucune.

## Tâches
| Lot | Tâche | Points | Fichiers | Fichier de référence |
|---|---|---|---|---|
| 1 | Toggler du daemon, états inactive/failed/absent, erreurs | 3 | src/app.rs, src/voxtype.rs | cosmic-applets/cosmic-applet-audio |
