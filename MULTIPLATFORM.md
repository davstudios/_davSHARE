# Supporto multipiattaforma — _davSHARE v26.10.1

## Desktop nativo

- Windows: Tauri 2 + WebView2.
- macOS: Tauri 2 + WKWebView.
- Linux: Tauri 2 + WebKitGTK.

## Client senza installazione

Android, iPhone, iPad, tablet e altri computer possono aprire una sessione `_davSHARE` da Safari, Chrome o un altro browser moderno scansionando il QR. Non servono APK o IPA.

## Rete

`_davSHARE` è LAN-only. Rileva gli IPv4 privati disponibili localmente senza usare servizi Internet, seleziona l'IP da pubblicare nel QR e avvia il server sulla porta 47821 o una delle successive disponibili. Il QR contiene IPv4, porta e token casuale.

Il server ascolta sulle interfacce locali, mentre token temporaneo e controllo della stessa subnet IPv4 limitano l'accesso alla sessione. La conferma manuale del dispositivo può essere attivata/disattivata nelle Impostazioni ed è attiva per impostazione predefinita.

La pagina browser mobile usa lo stesso stile della suite, Plus Jakarta Sans quando il webfont è raggiungibile e riceve quasi in tempo reale lo stato di chiusura della sessione desktop.

## Release

Il workflow GitHub costruisce e pubblica automaticamente installer NSIS per Windows, Universal DMG per macOS e AppImage/DEB per Linux quando viene pubblicato un tag `v*` coerente con la versione del progetto. La Description bilingue 🇮🇹/🇺🇸 del commit associato al tag viene utilizzata come descrizione della GitHub Release; il workflow interrompe la pubblicazione se una delle due sezioni manca.
