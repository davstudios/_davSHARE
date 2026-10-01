# BUILD NOTES — _davSHARE v26.10.1

Stack: Tauri 2.12, Rust 1.90+, JavaScript e Vite.

Backend: `tiny_http` per il server LAN, `qrcode` per il QR SVG, `zip` per la preparazione temporanea delle cartelle e `fs2` per il controllo dello spazio disponibile.

Il server ascolta sulle interfacce locali e pubblica nel QR l'IPv4 LAN selezionato. Ogni richiesta viene gestita in un worker separato. La sessione dura 10 minuti. Token, approvazioni client e trasferimenti attivi restano nel processo desktop. La chiusura della sessione invalida il token e i reader/writer verificano lo stato durante il trasferimento per interrompere i flussi non più validi.

Le cartelle da inviare vengono archiviate sotto la directory temporanea di sistema `_davSHARE/<token>` e ripulite quando la sessione viene sostituita, fermata o scade. I symlink vengono ignorati durante ispezione e creazione ZIP.

Per la ricezione vengono applicati limite massimo configurato, riserva minima di spazio libero, basename protetto, normalizzazione dei caratteri non validi e gestione dei nomi Windows riservati.

La cronologia backend è limitata agli ultimi 50 trasferimenti e il frontend ne conserva una copia locale in `localStorage`, contenente soltanto metadati.

La pagina browser mobile usa Plus Jakarta Sans tramite webfont e una CSP HTTP che autorizza esplicitamente `fonts.gstatic.com`. Il caricamento viene verificato con `FontFace`/`document.fonts` prima del rendering; se il font esterno non è raggiungibile viene usato il fallback senza bloccare i trasferimenti LAN.

## Versioni Tauri

- `tauri` Rust: 2.12.0
- `@tauri-apps/api`: 2.12.0
- `@tauri-apps/cli`: 2.12.0
- `tauri-build`: 2.7.0
- `tauri-plugin-dialog`: 2.7.3
- `tauri-plugin-opener`: 2.5.5

Le dipendenze core/plugin sono fissate a versioni precise per evitare mismatch tra frontend e backend.

## Icone

`src-tauri/icons/icon.ico` è l'asset definitivo fornito per `_davSHARE`; PNG e ICNS derivano dalla stessa sorgente.

## Release automatica

`.github/workflows/release.yml` si attiva sui tag `v*`, verifica che tag, `package.json`, `tauri.conf.json` e `Cargo.toml` abbiano la stessa versione, richiede nel commit associato al tag una Description contenente entrambe le sezioni 🇮🇹 e 🇺🇸, esegue i test e pubblica una GitHub Release stabile usando automaticamente quella Description come corpo della release:

- Windows: NSIS
- macOS: Universal DMG
- Linux: AppImage + DEB

Il job Linux disabilita preventivamente eventuali repository Microsoft presenti sul runner Ubuntu che possono restituire HTTP 403 pur non essendo necessari alla build Tauri.

L'ambiente usato per preparare il pacchetto non dispone di Cargo, quindi la compilazione Tauri nativa finale viene validata dai runner GitHub/ambiente di sviluppo.
