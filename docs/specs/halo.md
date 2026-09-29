# Halo de l'orbe

## Besoin
L'utilisateur dicte avec l'orbe à l'écran. Le flou du compositeur, approché par 24 bandes, montre des marches, un bord franc et des fentes où le fond passe (très visible sur fond vif). On le remplace par un voile sombre dessiné, lisse, qui suit la voix.

## Parcours
1. La dictée démarre : l'orbe apparaît en fondu avec son voile.
2. Pendant la parole : le voile s'étend et se densifie avec le niveau lissé du micro (même lissage que l'orbe).
3. Silence : il revient à sa taille de repos.
4. Fin : il disparaît en fondu avec l'orbe.

## Cas d'erreur et limites
- Pas de flux audio (socket absent) : niveau 0, voile au repos, comme l'orbe aujourd'hui.
- Transcription : même voile ; seuls les grains changent de couleur.
- Tailles S/M/L : le voile est proportionnel au rayon de l'orbe, jamais coupé par le bord de la surface.

## Hors-périmètre
- Flou réel de l'arrière-plan (supprimé, plus de requête `blur` au compositeur).
- Halo coloré, réglage du voile dans le popup.

## Critères d'acceptation
- [ ] Plus aucun appel au protocole de flou ni de découpage en bandes dans `src/overlay/`.
- [ ] Voile radial sombre (variante B de la maquette) : cœur ≈ 0,42 d'opacité, portée 1,9 × rayon au repos, jusqu'à 2,45 × rayon et 0,54 d'opacité au niveau max.
- [ ] Profil d'opacité continu et décroissant jusqu'à 0 au bord (test unitaire), sans marche visible sur fond vif au zoom ×2 de l'écran.
- [ ] Le voile tient dans la surface pour les 3 tailles (test unitaire).
- [ ] Le voile suit le fondu d'apparition et de disparition.

## Choix techniques
- Rendu : voile dessiné dans le canvas de l'orbe (tranché : « halo dessiné seul », variante B).
- Dégradé : iced n'a pas de dégradé radial ; anneaux concentriques fins (≈ 1 px) remplis en even-odd, opacité tirée d'une fonction de profil pure et testée.
- Voile dessiné même quand le thème n'a pas « interface givrée » (il ne dépend plus du compositeur).

## Hypothèses

## Questions en attente

## Tâches
| Lot | Tâche | Points | Fichiers | Fichier de référence |
|---|---|---|---|---|
| 1 | Fonction de profil du voile (portée, opacité selon distance et niveau) + tests | 2 | `src/overlay/view.rs` | `view.rs` (`radius`, `mix` et leurs tests) |
| 1 | Dessin du voile en anneaux dans `Orb::draw` (niveau passé à `Orb`), suppression du flou compositeur (`disc`, `BLUR_*`, `blur`) | 3 | `src/overlay/view.rs`, `src/overlay/mod.rs` | `Orb::draw` |
