# _davSHARE v26.10.3

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

## Installazione delle release GitHub non firmate

Le release di `_davSHARE` sono distribuite direttamente tramite GitHub e, al momento, non utilizzano certificati commerciali di code signing o notarizzazione Apple. Il codice sorgente è disponibile pubblicamente con licenza MIT.

### Windows

Windows SmartScreen può mostrare l'avviso **“Windows ha protetto il PC”** perché l'installer non è firmato con un certificato di publisher attendibile. Se hai scaricato il file dalla repository GitHub ufficiale di `_davstudios`, seleziona **Ulteriori informazioni** e poi **Esegui comunque**.

### macOS

Gatekeeper può impedire la prima apertura perché l'app non è firmata con Developer ID e non è notarizzata da Apple. Dopo aver tentato di aprire l'app, vai in **Impostazioni di Sistema → Privacy e Sicurezza**, individua il messaggio relativo a `_davSHARE` e scegli **Apri comunque**.

### Linux

Per un'AppImage può essere necessario rendere il file eseguibile prima dell'avvio:

```bash
chmod +x _davSHARE*.AppImage
```

Scarica sempre le release dalla repository GitHub ufficiale di `_davstudios`. Quando viene pubblicato un hash SHA-256, puoi usarlo per verificare l'integrità del file scaricato.

## Informazioni pacchetto

- Developer / Publisher: `_davstudios`
- Homepage: https://davstudios.it
- Copyright: © 2026 _davstudios
- Licenza: MIT
- Categoria: Productivity
- Bundle identifier: `studio.dav.share`
- Versione corrente: `26.10.3`

## Installing unsigned GitHub releases

`_davSHARE` releases are distributed directly through GitHub and currently do not use a commercial Windows code-signing certificate or Apple Developer ID notarization. The source code is publicly available under the MIT License.

### Windows

Windows SmartScreen may display **“Windows protected your PC”** because the installer is not signed by a trusted publisher certificate. If you downloaded the file from the official `_davstudios` GitHub repository, choose **More info** and then **Run anyway**.

### macOS

Gatekeeper may block the first launch because the app is not signed with Developer ID and notarized by Apple. After attempting to open the app, go to **System Settings → Privacy & Security**, find the `_davSHARE` message and choose **Open Anyway**.

### Linux

An AppImage may need to be marked as executable before launch:

```bash
chmod +x _davSHARE*.AppImage
```

Always download releases from the official `_davstudios` GitHub repository. When a SHA-256 hash is published, you can use it to verify the integrity of the downloaded file.

## Package information

- Developer / Publisher: `_davstudios`
- Homepage: https://davstudios.it
- Copyright: © 2026 _davstudios
- License: MIT
- Category: Productivity
- Bundle identifier: `studio.dav.share`
- Current version: `26.10.3`

## Avvio sviluppo

Windows: `RUN-WINDOWS.bat`

macOS: `./RUN-MACOS.sh`

Linux: prima `./INSTALL-LINUX-DEPS-UBUNTU.sh` se necessario, poi `./RUN-LINUX.sh`.

## Build

- Windows: `BUILD-WINDOWS.bat`
- macOS: `./BUILD-MACOS.sh`
- Linux: `./BUILD-LINUX.sh`

La GitHub Release stabile viene generata automaticamente dal workflow quando viene pubblicato un tag `v*` coerente con la versione dell'app.

