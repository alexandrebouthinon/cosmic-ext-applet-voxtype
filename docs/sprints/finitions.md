# Sprint finitions

Branche : `sprint/finitions`, depuis `develop`
Vélocité : 20 (petit sprint, hors backlog)

## Features
- Halo de l'orbe, voile sombre (docs/specs/halo.md) : 5
- Traduction française (docs/specs/i18n-fr.md) : 4

Pas de lot 0 : aucun contrat partagé entre les deux features.

Livrable testable : l'orbe s'affiche avec un voile lisse qui suit la voix, sans bandes ; avec `LANG=fr_FR.UTF-8`, l'applet est en français.

Maquette : https://claude.ai/artifact/CYw8gJCW6DM68gC6fsGHab (variante B).

## À capitaliser

## Recette
| # | Retour | Type | Branche | Points | Statut |
|---|---|---|---|---|---|
| 1 | Onglet Microphone trop long : « Micro » / « Mic » | ajustement | feat/i18n-fr | 1 | validé |
| 2 | Libellé de l'interrupteur de l'orbe trop long, pousse le bouton hors écran : « Activer » / « Enable » | ajustement | feat/i18n-fr | 1 | validé |

## Bilan (fin de sprint)
- Points prévus / livrés : 9 / 9 (+2 en recette)
- Marché : maquette en 3 variantes, voile choisi du premier coup ; voile en disques emboîtés, lisse et sans couture.
- Raté → règle :
  - Libellés FR trop longs (onglet, interrupteur de l'orbe) et « Backend » traduit à tort → libellés courts, termes techniques non traduits (piège `archi`)
- Capitalisé : `tech/cosmic/libcosmic-layer-surface-rendering` (voile radial dessiné sous iced), `tech/cosmic/cosmic-panel-applet-overlay` (traduction du `.desktop` par xdgen)
