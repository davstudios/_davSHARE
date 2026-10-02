# Changelog

## 26.10.2

- Corretto il test di sincronizzazione della versione su Windows: `Cargo.lock` viene ora letto correttamente sia con terminatori LF sia CRLF.
- Aggiunto un test di regressione che simula esplicitamente un checkout Windows con `Cargo.lock` in CRLF.
- Rafforzato `.gitattributes` per mantenere gli script shell e `src-tauri/Cargo.lock` con terminatori LF nei checkout futuri.
- Sincronizzata la versione tecnica e di release a `26.10.2`.
- Nessuna modifica apportata al motore di condivisione LAN, ai trasferimenti, alle sessioni QR, alla sicurezza di rete, all'interfaccia o alla logica funzionale dell'app.

## 26.10.1

- Adottato il nuovo standard di release `_davstudios` e il sistema di versioning `YY.M.REVISIONE`.
- Sincronizzata la versione in `package.json`, `package-lock.json`, `Cargo.toml`, `Cargo.lock`, configurazione Tauri, documentazione e test.
- Standardizzati i metadata ufficiali con publisher `_davstudios`, homepage `davstudios.it`, copyright © 2026 `_davstudios`, licenza MIT, categoria Productivity e metadata Debian per Linux.
- Mantenuto invariato l'identifier storico `studio.dav.share`.
- Aggiunte al README le istruzioni per release non firmate su Windows SmartScreen, macOS Gatekeeper e Linux AppImage.
- Il workflow GitHub Actions usa la Description bilingue 🇮🇹/🇺🇸 del commit associato al tag come descrizione della GitHub Release e ne verifica la presenza prima della pubblicazione.
- Mantenuto l'hardening Linux contro repository Microsoft non raggiungibili sui runner Ubuntu.
- Nessuna modifica apportata al motore di condivisione LAN, ai trasferimenti, alle sessioni QR, alla sicurezza di rete, all'interfaccia o alla logica funzionale dell'app.

## 1.0.1

- Patch release di `_davSHARE` per riallineare la versione dopo un errore durante il push della v1.0.0.
- Nessuna modifica funzionale o grafica rispetto alla v1.0.0.
- Versioni tecniche, UI, documentazione, test e metadati di release sincronizzati a 1.0.1.

## 1.0.0

- Prima release stabile di `_davSHARE`.
- Consolidato il trasferimento bidirezionale desktop ↔ browser mobile tramite QR sulla stessa LAN.
- Stabilizzati trasferimenti concorrenti, progresso byte-level, annullamento reale della sessione e conferma dispositivo.
- Supporto stabile a file multipli e cartelle mediante ZIP temporanei.
- Consolidati controlli su spazio disponibile, limiti upload, nomi file e percorsi.
- Pagina mobile IT/EN allineata al design `_davstudios`, con Plus Jakarta Sans e chiusura sessione live.
- Stabilizzati diagnostica rete, scelta IP LAN, cronologia locale e launcher Windows.
- Aggiunto workflow GitHub Release stabile per Windows, macOS e Linux con verifica automatica della versione del tag.

# Changelog

## 0.4.3 Preview

- Corretto il vero blocco del font mobile: la Content Security Policy del server LAN non autorizzava `fonts.gstatic.com`, quindi Safari/Chrome ricadevano sul font di sistema anche se `@font-face` era presente.
- Aggiunto `font-src https://fonts.gstatic.com` alla CSP delle pagine mobili servite da `_davSHARE`.
- Il browser mobile carica ora Plus Jakarta Sans esplicitamente tramite `FontFace`, lo registra in `document.fonts` e verifica il caricamento prima di mostrare l’interfaccia.
- Motore di trasferimento e chiusura sessione live invariati rispetto alla v0.4.2.

## 0.4.2 Preview

- Desktop e pagina mobile ora caricano Plus Jakarta Sans dalla stessa risorsa webfont tramite `@font-face`; la pagina mobile usa anche il preload.
- Rimossa la dipendenza dal foglio CSS remoto di Google Fonts, che su alcuni browser mobili poteva lasciare attivo il font di sistema.
- La pagina attende i pesi Regular ed ExtraBold prima del rendering, con fallback temporizzato per non bloccare mai i trasferimenti LAN.
- Disattivato il ridimensionamento automatico del testo di Safari per mantenere la resa tipografica più vicina al desktop.


## 0.4.1 Preview

- Tipografia mobile riallineata alla suite con Plus Jakarta Sans su Safari/Chrome mobile.
- Aggiunto monitoraggio live dello stato sessione dal browser: la chiusura sul desktop mostra automaticamente “Sessione terminata” sul telefono.
- Gli upload XHR attivi vengono annullati lato browser quando la sessione viene chiusa.
- Rifinita anche la pagina di sessione scaduta con lo stesso stile mobile.

## 0.3.5 Preview

- Corretto il launcher Windows per chiudere in sicurezza vecchie sessioni Vite/Tauri di `_davSHARE` prima di `npm install` e `tauri dev`.
- La porta di sviluppo 17462 viene verificata prima dell'avvio; processi estranei non vengono terminati automaticamente.
- Il cleanup della sessione precedente viene eseguito prima della sincronizzazione npm per evitare lock/EPERM sui moduli nativi Tauri.

## 0.3.3 Preview

- Allineati Tauri Rust, `@tauri-apps/api` e CLI al ramo 2.12 per eliminare il mismatch tra core Rust e frontend JavaScript.
- Fissate le versioni dei plugin Tauri usati dal progetto per evitare aggiornamenti incoerenti.

## 0.3.2 Preview

- Aggiornata l'icona definitiva di `_davSHARE` usando il file `icon.ico` fornito.
- Rigenerati tutti gli asset Tauri necessari per Windows, macOS e Linux a partire dalla stessa icona.
- Nessuna modifica funzionale rispetto alla v0.3.1.

## 0.3.1 Preview

- Primo tentativo di allineamento delle versioni Tauri per eliminare il mismatch rilevato in sviluppo.
- Aggiunto un test di contratto per impedire future divergenze tra il core Tauri Rust e `@tauri-apps/api`.

## 0.3.0 Preview

- Motore LAN rivisto: rilevamento IPv4 dalle interfacce locali senza dipendenza da Internet.
- Server HTTP associato allo specifico IP LAN invece di `0.0.0.0`.
- Gestione concorrente delle richieste HTTP per trasferimenti simultanei.
- Progresso byte-level per upload e download con percentuale, velocità media e tempo rimanente stimato.
- Chiusura sessione resa effettiva anche sui trasferimenti in corso.
- Aggiunta conferma opzionale del dispositivo con Accetta/Rifiuta dal desktop.
- Aggiunto supporto alle cartelle mediante ZIP temporaneo automatico.
- Aggiunti controllo spazio libero e limite upload configurabile.
- Rafforzata la sanificazione dei nomi file e aggiunto `filename*` UTF-8 nei download.
- Pagina mobile resa bilingue IT/EN e upload mobile con progresso reale.
- Aggiunta cronologia locale degli ultimi 50 trasferimenti.
- Aggiunta diagnostica rete con IP, porta, interfacce e self-test.
- UI mantenuta coerente con il design system `_davUNINSTALL`.

## 0.2.0 Preview

- UI riallineata direttamente a `_davUNINSTALL` v1.1.1.
- Copiati token visuali, colori, tipografia, sidebar, pulsanti, menu, impostazioni e sistema di animazioni della suite.
- Mantenute modalità Invia/Ricevi tramite server LAN e QR temporaneo.

## 0.1.0 Preview

- Prima base funzionante desktop + browser mobile.
- Invio e ricezione tramite QR temporaneo sulla LAN.
