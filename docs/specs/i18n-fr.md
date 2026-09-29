# French translation

## Need
The user has a French-language system: the applet must speak French to them, and English to everyone else.

## Flow
1. On startup, the applet reads the system language (`LANG`, `LC_*` via `DesktopLanguageRequester`, already in place).
2. Language `fr` regardless of the variant (fr_FR, fr_CA, fr_BE…): everything in French.
3. Any other language: English.
4. Name, description and keywords in panel settings and the app center: translated via the generated desktop and metainfo files.

## Edge cases and limits
- Key missing from the French file: falls back to English for that key (never a raw identifier); a test prevents shipping one.
- Short labels to fit in the popup: "Micro" / "Mic" tab, orb switch "Activer" / "Enable" (fixed label).
- Technical messages returned by Voxtype or `pactl`/`wpctl`: displayed as-is, not translated.
- System language change: taken into account on the applet's next launch.

## Out of scope
- Languages other than fr and en.
- Language choice in the popup.

## Acceptance criteria
- [ ] `i18n/fr/cosmic_ext_applet_voxtype.ftl` contains all the English keys (parity test).
- [ ] `fr_CA` and `fr_FR` give French, `de_DE` and `C` give English (selection test).
- [ ] The generated `.desktop` file contains `Name[fr]`, `Comment[fr]`, `Keywords[fr]`, and the metainfo its `summary` in fr.
- [ ] With `LANG=fr_FR.UTF-8`, popup, errors and tabs in French on the panel.
- [ ] No label pushes a control out of the popup, in French or English.

## Technical choices
- Detection: `DesktopLanguageRequester` + i18n-embed negotiation, fallback `en` (existing, decided by the user: any fr_* locale).
- Scope: applet + desktop and metainfo files (decided by the user).

## Assumptions

## Open questions

## Tasks
| Batch | Task | Points | Files | Reference file |
|---|---|---|---|---|
| 1 | Complete French file, verified in the generated `.desktop` and metainfo | 2 | `i18n/fr/cosmic_ext_applet_voxtype.ftl` | `i18n/en/cosmic_ext_applet_voxtype.ftl` |
| 1 | Key parity and language selection tests | 2 | `src/i18n.rs` | `src/i18n.rs` |
