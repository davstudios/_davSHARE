# _davSHARE v1.0.1

🇮🇹 `_davSHARE` è l'app desktop di _davstudios per condividere file e cartelle nella rete locale senza cloud. L'app gira su Windows, macOS e Linux; Android, iPhone, iPad e altri dispositivi accedono dal browser tramite un QR temporaneo, senza installare APK o IPA.

🇺🇸 `_davSHARE` is the _davstudios desktop app for sharing files and folders over the local network without cloud storage. The app runs on Windows, macOS and Linux; Android, iPhone, iPad and other devices connect from a browser through a temporary QR code, with no APK or IPA required.

## Funzioni principali

- Invio desktop → browser mobile di file multipli e cartelle.
- Ricezione browser mobile → desktop con progresso reale in byte.
- Cartelle preparate automaticamente come ZIP temporanei e rimosse alla chiusura/rigenerazione della sessione.
- Server HTTP locale concorrente per gestire più richieste contemporaneamente.
- QR generato localmente con URL `http://IP-LOCALE:PORTA/s/TOKEN`.
- Token casuale e scadenza automatica della sessione dopo 10 minuti.
- Chiusura sessione propagata quasi in tempo reale al browser mobile, con blocco/annullamento dei trasferimenti attivi.
- Conferma dispositivo opzionale con Accetta/Rifiuta sul desktop.
- Pagina mobile IT/EN nello stesso stile della suite e con Plus Jakarta Sans.
- Stato desktop con byte trasferiti, percentuale, velocità media e tempo rimanente stimato.
- Limite massimo per file ricevuto configurabile: 1/5/10/25/100 GB.
- Controllo dello spazio libero con riserva di sicurezza.
- Sanificazione cross-platform dei nomi file e protezione dei percorsi.
- Cronologia locale degli ultimi 50 trasferimenti, composta soltanto da metadati.
- Diagnostica rete con IP, porta, interfacce rilevate e self-test.
- UI, colori, sidebar, impostazioni, dropdown, pulsanti e motion system coerenti con la suite `_davstudios`.

## Requisiti di rete

Computer e dispositivo mobile devono potersi raggiungere sulla stessa LAN/Wi-Fi. `_davSHARE` filtra le richieste sulla stessa subnet IPv4 della sessione. Reti Guest con isolamento client, firewall restrittivi o VPN possono impedire la connessione. Su Windows, al primo utilizzo del server locale, consentire l'accesso di `_davSHARE` almeno sulle reti private.

La condivisione dei file è LAN-only e non usa cloud. Plus Jakarta Sans nella pagina mobile viene caricato come webfont esterno; se il telefono non ha connettività Internet o il dominio del font è bloccato, il trasferimento continua a funzionare ma il browser può usare il font di sistema come fallback.

## Avvio sviluppo

Windows: `RUN-WINDOWS.bat`

macOS: `./RUN-MACOS.sh`

Linux: prima `./INSTALL-LINUX-DEPS-UBUNTU.sh` se necessario, poi `./RUN-LINUX.sh`.

## Build

- Windows: `BUILD-WINDOWS.bat`
- macOS: `./BUILD-MACOS.sh`
- Linux: `./BUILD-LINUX.sh`

La GitHub Release stabile viene generata automaticamente dal workflow quando viene pubblicato un tag `v*` coerente con la versione dell'app.
