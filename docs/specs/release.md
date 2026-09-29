# Branche develop et releases

## Besoin
Publier des versions propres de l'applet (tag, notes, binaire) sans travail manuel, et installer l'applet sans cloner le repo.

## Parcours
1. Le travail arrive sur `develop` ; `just release` y publie une pré-release (`vX.Y.Z-develop.N`) : tag, notes, `Cargo.toml`, `Cargo.lock` et CHANGELOG commités, sans binaire.
2. `develop` mergé sur `main` en fast-forward ; `just release` publie la version stable : tag, GitHub Release, CHANGELOG, bump de `Cargo.toml`, binaire x86_64 compilé en local et attaché.
3. L'utilisateur télécharge le binaire et lance `cosmic-ext-applet-voxtype --install` : copie dans `~/.local/bin`, écrit le `.desktop` (avec `X-CosmicApplet=true` et `Exec=` absolu) et l'icône dans `~/.local/share`. `--uninstall` retire les trois.
4. GitHub Actions lance `just verify` sur chaque push et PR.

## Cas d'erreur et limites
- `just release` hors de `main`/`develop`, ou arbre sale : refus avec message.
- Pas de commit `feat`/`fix` depuis la dernière version : pas de release, message de semantic-release.
- `--install` quand `~/.local/bin` n'existe pas : le créer. Fichier déjà présent : remplacé.

## Hors-périmètre
- AppImage, Flatpak (le sandbox bloque la CLI voxtype, systemctl et `audio.sock`), deb, rpm.
- Builds multi-arch en CI (quand le repo sera public).

## Critères d'acceptation
- [ ] La branche `develop` existe sur le remote et `CLAUDE.md` décrit le flux.
- [ ] `just release` sur `develop` crée une pré-release sans binaire.
- [ ] `just release` sur `main` crée une release avec le binaire attaché.
- [ ] Le binaire téléchargé, avec `--install`, apparaît dans COSMIC Settings → Panel ; `--uninstall` le retire.
- [ ] La CI échoue si `just verify` échoue.

## Choix techniques
- semantic-release + semantic-release-cargo, lancé en local via `just release` (tranché par l'utilisateur).
- Binaire seul avec `--install`/`--uninstall`, `.desktop` et icône embarqués (tranché par l'utilisateur).
- Pré-releases sans binaire, x86_64 seulement (tranché par l'utilisateur).

## Hypothèses
- Aucune.

## Tâches
| Lot | Tâche | Points | Fichiers | Fichier de référence |
|---|---|---|---|---|
| 1 | Branche develop, release.config.mjs, package.json, recette `just release`, CHANGELOG | 3 | release.config.mjs, package.json, justfile, CLAUDE.md | — |
| 1 | `--install` / `--uninstall` (desktop + icône embarqués) | 3 | src/main.rs, src/install.rs | justfile (recette install-user) |
| 1 | Workflow CI `just verify` | 1 | .github/workflows/verify.yml | — |
