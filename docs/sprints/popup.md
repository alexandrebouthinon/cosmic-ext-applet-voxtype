# Sprint popup

Branche : `sprint/popup`, depuis `develop` (créée depuis `main` au lancement)
Vélocité : 20

## Features
- Popup : en-tête + onglets Statut et Orbe (docs/specs/applet.md) : 7 (lot 0 : 3, lot 1 : 4)
- Choix du micro, onglet Micro (docs/specs/micro.md) : 3
- Arrêt et relance du daemon (docs/specs/daemon.md) : 3
- Branche develop et releases (docs/specs/release.md) : 7

Livrable testable : popup à onglets dans le panneau ; couper le toggler libère le GPU ; `just release` publie une pré-release ; le binaire s'installe avec `--install`.

Maquette : https://claude.ai/artifact/Twxfwdprz56zxNrgHM3Rqp (disposition B).

## À capitaliser
- Voxtype `audio.device` n'accepte que les noms ALSA de `voxtype info devices` ; un nom de source PipeWire (`alsa_input.…`) donne `Audio device not found` au démarrage de l'enregistrement, pas au démarrage du daemon (test réel)
- Le sandbox `workspace-write` de Codex ne peut pas lancer cargo : la vérification des tâches Codex revient à Claude (sprint popup)
- libcosmic : `Subscription::run_with(data, f)` recrée la subscription quand `data` change ; un compteur de génération force la reconnexion d'un suiveur (`voxtype status --follow`) après un start, `kill_on_drop` tue l'ancien enfant (daemon-1)
- semantic-release-cargo : `prepare(pluginConfig, context)` lit `context.nextRelease.version` ; lui passer la version seule lève un TypeError (release-2)
- Plusieurs features qui ajoutent des champs/messages à `AppModel` passent chacune clippy seules mais pas ensemble (`struct_excessive_bools`, `too_many_lines`) : regrouper l'état par onglet (struct) et un `update_<feature>` par feature dès le brief (intégration lot 1)

## Recette
| # | Retour | Type | Branche | Points | Statut |
|---|---|---|---|---|---|
| 1 | Onglet Statut : infos correctes | — | — | — | validé |
| 2 | Toggle daemon : arrête bien Parakeet | — | — | — | validé |
| 3 | Micro : liste ALSA brute et illisible ; lister les sources PipeWire (nom lisible, sans monitors) et choisir via `pactl set-default-source`, Voxtype sur `default` | ajustement | feat/micro | 3 | validé |
| 4 | Orbe : l'indicateur de sélection masque les libellés S/M/L | ajustement | feat/applet | 1 | validé |

## Bilan (fin de sprint)
- Points prévus / livrés : …
- Rétro : …
