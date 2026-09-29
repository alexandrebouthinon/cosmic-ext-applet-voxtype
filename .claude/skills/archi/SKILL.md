---
name: archi
description: "Use when writing a brief for a subagent, Codex or Copilot, or before coding in an unfamiliar module of cosmic-ext-applet-voxtype: modules, where new code goes, reference files, pitfalls."
---

# Architecture de cosmic-ext-applet-voxtype

Un seul binaire, deux processus : l'applet (lancée par cosmic-panel) et l'overlay (`--overlay`, enfant de l'applet). `src/main.rs` choisit selon l'argument.

## Modules
- `src/app.rs` : applet du panneau (icône, popup, toggle overlay). Lance et tue l'enfant `--overlay` (`OverlayChild`, kill + wait au Drop). Dépend de `voxtype`, `config`.
- `src/overlay/mod.rs` : app libcosmic sans fenêtre principale ; crée/détruit la layer surface click-through selon l'état Voxtype, fade, blur en disque. Quitte si `parent_id()` change.
- `src/overlay/particles.rs` : simulation pure et déterministe (seed), sans iced. Testable seule.
- `src/overlay/audio.rs` : lecture de `$XDG_RUNTIME_DIR/voxtype/audio.sock` (trames 16 octets) + lissage.
- `src/overlay/view.rs` : dessin canvas et palette depuis le thème COSMIC.
- `src/voxtype.rs` : client CLI (`voxtype status --follow --format json`), `State`, `Event`. Partagé par les deux processus.
- `src/config.rs` : cosmic-config (`show_overlay`, position, taille), partagé ; l'overlay lit l'APP_ID de l'applet.

## Où mettre du nouveau code
| Type | Emplacement | Fichier de référence |
|---|---|---|
| UI du panneau / popup | `src/app.rs` | cosmic-applets `cosmic-applet-audio` |
| Réglage persistant | `src/config.rs` (+ bump `Config::VERSION` si rupture) | `src/config.rs` |
| Appel à Voxtype | `src/voxtype.rs`, CLI seulement | `parse_status_line` + ses tests |
| Logique d'animation | `src/overlay/particles.rs` (pure, testée) | `src/overlay/particles.rs` |
| Rendu | `src/overlay/view.rs` | `src/overlay/view.rs` |
| Texte affiché | `i18n/en/*.ftl` + `fl!` | `src/app.rs` |

## Conventions
- Logique pure hors des `update`/`view`, avec tests unitaires dans le même fichier.
- Subscriptions coûteuses (tick 60 fps, socket audio) actives seulement quand la surface existe : 0 % CPU au repos.
- Couleurs toujours tirées du thème COSMIC actif, jamais codées en dur.
- Clippy pedantic en `-D warnings` : exceptions par `#[expect(..., reason = ...)]`.

## Pièges
- Jamais de layer surface dans l'applet : cosmic-panel les proxifie. L'overlay est un processus séparé, avec `X-HostWaylandDisplay=true` dans le `.desktop`.
- libcosmic floute toute la surface par défaut : `core.set_auto_blur(Default::default())`, puis `blur()` avec des rectangles.
- Tests panneau : `just install-user` puis `pkill -x cosmic-panel` (relancé par la session). `Exec=` absolu obligatoire.
- `pkill -f` : ancrer le motif (`'^[^ ]*cosmic-ext-applet-voxtype --overlay$'`), sinon il tue son propre shell.
- `just run` n'est pas le rendu du panneau ; l'overlay seul : `cargo run -- --overlay` (quitte si son parent meurt).
- Voxtype : CLI uniquement, jamais modifié ni forké.
- Listes système (micros, sorties…) : valider en cadrage sur les données réelles de la machine, face à ce qu'affiche COSMIC. Voxtype ne voit que les PCM ALSA ; les micros passent par PipeWire (`src/pipewire.rs`), Voxtype restant sur `default`.
- `AppModel` : l'état de chaque onglet dans sa propre struct et un `update_<feature>` par feature, dès le brief. Sinon, chaque feature passe clippy seule mais plus une fois réunies (`struct_excessive_bools`, `too_many_lines`).
