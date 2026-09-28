# Sprint orbe

Branche : `sprint/orbe`
Vélocité : 20

## Features
- Overlay « orbe de particules » (docs/specs/overlay.md) : 18 (lot 0 : 4, lot 1 : 14)
- Icône du panneau selon l'état (docs/specs/applet.md, tâche lot 1) : 2

Livrable testable : applet dans le panneau, icône à jour, orbe 10b qui apparaît pendant la dictée et réagit à la voix.

## À capitaliser
- L'OSD gtk4 de Voxtype crashe sur cosmic-comp 1.9 : au premier masquage, il détruit la `zwlr_layer_surface`, puis fait un attach nil + commit, et le compositeur le déconnecte (cadrage).
- Niveau micro de Voxtype : `$XDG_RUNTIME_DIR/voxtype/audio.sock`, trames de 16 octets à 100 Hz, ordre natif (seq u32, min f32, max f32, peak_dbfs f32, borné à -120) (voxtype src/osd/ipc.rs).
- `voxtype config set audio.device` ne s'applique qu'après `systemctl --user restart voxtype` (cadrage).

## Bilan (fin de sprint)
- Points prévus / livrés : …
- Rétro : …
