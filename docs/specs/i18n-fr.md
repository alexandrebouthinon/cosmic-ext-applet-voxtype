# Traduction française

## Besoin
L'utilisateur a un système en français : l'applet doit lui parler français, et en anglais à tous les autres.

## Parcours
1. Au démarrage, l'applet lit la langue du système (`LANG`, `LC_*` via `DesktopLanguageRequester`, déjà en place).
2. Langue `fr` quelle que soit la variante (fr_FR, fr_CA, fr_BE…) : tout en français.
3. Toute autre langue : anglais.
4. Nom, description et mots-clés dans les réglages du panneau et le centre d'applications : traduits via les fichiers desktop et metainfo générés.

## Cas d'erreur et limites
- Clé absente du fichier français : repli sur l'anglais pour cette clé (jamais d'identifiant brut) ; un test empêche d'en livrer.
- Messages techniques renvoyés par Voxtype ou `pactl`/`wpctl` : affichés tels quels, non traduits.
- Changement de langue système : pris en compte au prochain lancement de l'applet.

## Hors-périmètre
- Autres langues que fr et en.
- Choix de la langue dans le popup.

## Critères d'acceptation
- [ ] `i18n/fr/cosmic_ext_applet_voxtype.ftl` contient toutes les clés de l'anglais (test de parité).
- [ ] `fr_CA` et `fr_FR` donnent le français, `de_DE` et `C` l'anglais (test de sélection).
- [ ] Le `.desktop` généré contient `Name[fr]`, `Comment[fr]`, `Keywords[fr]`, et le metainfo son `summary` en fr.
- [ ] Avec `LANG=fr_FR.UTF-8`, popup, erreurs et onglets en français au panneau.

## Choix techniques
- Détection : `DesktopLanguageRequester` + négociation d'i18n-embed, repli `en` (existant, tranché : toute locale fr_*).
- Périmètre : applet + fichiers desktop et metainfo (tranché).

## Hypothèses

## Questions en attente

## Tâches
| Lot | Tâche | Points | Fichiers | Fichier de référence |
|---|---|---|---|---|
| 1 | Fichier français complet, vérifié dans le `.desktop` et le metainfo générés | 2 | `i18n/fr/cosmic_ext_applet_voxtype.ftl` | `i18n/en/cosmic_ext_applet_voxtype.ftl` |
| 1 | Tests de parité des clés et de sélection de la langue | 2 | `src/i18n.rs` | `src/i18n.rs` |
