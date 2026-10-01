import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';

const main=fs.readFileSync('src/main.js','utf8');
const css=fs.readFileSync('src/styles.css','utf8');
const motion=fs.readFileSync('src/motion.css','utf8');
const rust=fs.readFileSync('src-tauri/src/lib.rs','utf8');
const cargo=fs.readFileSync('src-tauri/Cargo.toml','utf8');
const tauri=JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json','utf8'));

test('design system suite presente',()=>{assert.match(css,/--accent:#006edb/);assert.match(css,/--bg:#0b0b0d/);assert.match(css,/Plus Jakarta Sans/);assert.match(main,/Comprami Un Caffè/);assert.match(main,/davstudios\.it/);});
test('desktop e mobile caricano la stessa risorsa Plus Jakarta Sans',()=>{assert.match(css,/fonts\.gstatic\.com\/s\/plusjakartasans/);assert.match(rust,/fonts\.gstatic\.com\/s\/plusjakartasans/);assert.match(tauri.app.security.csp,/font-src 'self' https:\/\/fonts\.gstatic\.com/);});
test('navigazione desktop include quattro sezioni',()=>{for(const page of ["'send'","'receive'","'activity'","'settings'"])assert.match(main,new RegExp(page));});
test('server locale usa porte dedicate _davSHARE',()=>{assert.match(rust,/DEFAULT_PORT:u16=47821/);assert.equal(tauri.build.devUrl,'http://localhost:17462');});
test('sessioni hanno token casuale e scadenza',()=>{assert.match(rust,/Alphanumeric/);assert.match(rust,/SESSION_TTL_SECONDS:u64=600/);assert.match(rust,/expires_at/);});
test('QR viene generato localmente in Rust',()=>{assert.match(cargo,/qrcode/);assert.match(rust,/QrCode::new/);assert.match(rust,/render::<svg::Color>/);});
test('mobile supporta download desktop verso browser',()=>{assert.match(rust,/\/download\//);assert.match(rust,/Content-Disposition/);assert.match(main,/Un QR, nessuna app sul telefono/);});
test('mobile supporta upload browser verso desktop',()=>{assert.match(rust,/values\[2\]=="upload"/);assert.match(rust,/safe_filename/);assert.match(rust,/unique_destination/);assert.match(main,/Ricevi dal telefono/);});
test('upload non può scegliere un percorso arbitrario',()=>{assert.match(rust,/file_name\(\)/);assert.match(rust,/session\.receive_dir/);assert.match(rust,/CON.*PRN.*AUX.*NUL/);});
test('server ascolta sulle interfacce locali e non dipende da Internet',()=>{assert.match(rust,/Server::http\(format!\("0\.0\.0\.0:\{port\}"\)\)/);assert.match(rust,/local_ipv4_candidates/);assert.doesNotMatch(rust,/8\.8\.8\.8/);});
test('server gestisce richieste concorrenti',()=>{assert.match(rust,/davshare-transfer/);assert.match(rust,/spawn\(move\|\|handle_request/);});
test('trasferimenti hanno progresso byte-level',()=>{assert.match(rust,/TransferProgress/);assert.match(rust,/transferred/);assert.match(main,/data-transfer-progress/);assert.match(main,/formatBytes\(done\)/);});
test('stop sessione interrompe i flussi attivi',()=>{assert.match(rust,/session_is_active/);assert.match(rust,/io::ErrorKind::Interrupted/);assert.match(rust,/remove_file\(&target\)/);});
test('conferma dispositivo è opzionale e gestita dal desktop',()=>{assert.match(rust,/require_confirmation/);assert.match(rust,/approve_client/);assert.match(rust,/reject_client/);assert.match(main,/requireConfirmation/);});
test('ispezione cartelle gira fuori dal thread UI',()=>{assert.match(rust,/async fn inspect_share_files/);assert.match(rust,/spawn_blocking\(move\|\|inspect_paths\(paths\)\)/);});
test('cartelle vengono preparate come ZIP temporanei',()=>{assert.match(cargo,/zip = "0\.6"/);assert.match(rust,/ZipWriter/);assert.match(rust,/prepare_share_files/);assert.match(main,/Aggiungi cartella/);});
test('spazio libero e limite upload vengono controllati',()=>{assert.match(cargo,/fs2 = "0\.4"/);assert.match(rust,/available_space/);assert.match(rust,/MIN_FREE_RESERVE_BYTES/);assert.match(main,/uploadLimitGb/);});
test('download conserva anche i nomi UTF-8',()=>{assert.match(rust,/filename\*=UTF-8''/);assert.match(rust,/rfc5987_filename/);});
test('pagina mobile segue italiano o inglese della sessione',()=>{assert.match(rust,/session\.language=="en"/);assert.match(main,/language:state\.settings\.language/);});
test('cronologia locale mantiene fino a 50 trasferimenti',()=>{assert.match(rust,/HISTORY_LIMIT:usize=50/);assert.match(main,/davshare-history/);assert.match(main,/slice\(0,50\)/);});
test('diagnostica rete è disponibile nell’app',()=>{assert.match(rust,/network_diagnostics/);assert.match(main,/Verifica rete/);assert.match(main,/Self-test/);});
test('drag and drop desktop è attivo anche per cartelle',()=>{assert.equal(tauri.app.windows[0].dragDropEnabled,true);assert.match(main,/onDragDropEvent/);assert.match(rust,/metadata\.is_dir\(\)/);});
test('motion della suite è presente',()=>{assert.match(motion,/dav-page-in/);assert.match(motion,/dav-qr-in/);assert.match(motion,/dav-live/);assert.match(motion,/live-transfer/);});
test('dipendenze Tauri core e plugin sono fissate a versioni coerenti',()=>{const pkgText=fs.readFileSync('package.json','utf8');assert.match(pkgText,/@tauri-apps\/api"\s*:\s*"2\.12\.0"/);assert.match(pkgText,/@tauri-apps\/cli"\s*:\s*"2\.12\.0"/);assert.match(pkgText,/@tauri-apps\/plugin-dialog"\s*:\s*"2\.7\.3"/);assert.match(pkgText,/@tauri-apps\/plugin-opener"\s*:\s*"2\.5\.5"/);assert.match(cargo,/tauri\s*=\s*\{\s*version\s*=\s*"=2\.12\.0"/);assert.match(cargo,/tauri-build\s*=\s*\{\s*version\s*=\s*"=2\.7\.0"/);assert.match(cargo,/tauri-plugin-dialog\s*=\s*"=2\.7\.3"/);assert.match(cargo,/tauri-plugin-opener\s*=\s*"=2\.5\.5"/);assert.match(cargo,/rust-version\s*=\s*"1\.90"/);});
test('sessione limita i client alla stessa subnet LAN',()=>{assert.match(rust,/fn same_lan/);assert.match(rust,/a\[0\]==b\[0\].*a\[1\]==b\[1\].*a\[2\]==b\[2\]/);});
test('upload incompleto viene eliminato',()=>{assert.match(rust,/expected>0&&expected!=total/);assert.match(rust,/Trasferimento incompleto/);assert.match(rust,/remove_file\(&target\)/);});
test('UI usa direttamente il design system _davUNINSTALL',()=>{assert.match(css,/--radius-sm:8px;--radius-md:12px;--radius-lg:20px/);assert.match(css,/\.shell\{display:grid;grid-template-columns:220px 1fr;height:100vh\}/);assert.match(css,/\.coffee-button\{min-width:0;flex:1;height:34px/);assert.match(css,/\.website-button\{min-width:0;flex:1;height:39px/);assert.match(css,/\.dav-select-trigger\{width:100%;min-height:38px/);assert.match(main,/theme-icon theme-icon-sun/);assert.match(main,/theme-icon theme-icon-moon/);});
test('impostazioni SHARE mantengono il layering dei menu della suite',()=>{assert.match(main,/settings-grid share-settings-page/);assert.match(css,/share-settings-page \.settings-card\{position:relative;z-index:5\}/);assert.match(css,/setting-control:has\(\.dav-select\.is-open\)\{z-index:30\}/);});

test('Windows launcher libera vecchie sessioni _davSHARE prima di npm install',()=>{const run=fs.readFileSync('RUN-WINDOWS.bat','utf8');const prep=fs.readFileSync('scripts/prepare-windows-dev.ps1','utf8');assert.match(run,/prepare-windows-dev\.ps1" -Port 17462/);assert.ok(run.indexOf('prepare-windows-dev.ps1')<run.indexOf('npm install --no-audit --no-fund'));assert.match(prep,/Get-CimInstance Win32_Process/);assert.match(prep,/\$commandLine -like "\*\$RepoRoot\*"/);assert.match(prep,/taskkill\.exe \/PID \$ProcessId \/T \/F/);assert.match(prep,/Get-NetTCPConnection -LocalPort \$Port -State Listen/);});
test('Windows launcher non passa RepoRoot quotato con slash finale',()=>{const run=fs.readFileSync('RUN-WINDOWS.bat','utf8');assert.doesNotMatch(run,/-RepoRoot\s+"%~dp0"/);});

test('Windows preferisce adattatori fisici attivi per il QR',()=>{assert.match(rust,/HardwareInterface -eq \$true/);assert.match(rust,/InterfaceMetric/);});
test('utente può selezionare manualmente l IP LAN del QR',()=>{assert.match(main,/networkIp/);assert.match(main,/Rete locale/);assert.match(rust,/network_ip:String/);assert.match(rust,/candidates.contains\(&parsed\)/);});

test('pagina mobile usa Plus Jakarta Sans come la suite desktop',()=>{assert.match(rust,/@font-face/);assert.match(rust,/fonts\.gstatic\.com\/s\/plusjakartasans/);assert.match(rust,/font-weight:200 800/);assert.match(rust,/font-family:"Plus Jakarta Sans"/);assert.match(rust,/body,button,input/);assert.doesNotMatch(rust,/fonts\.googleapis\.com/);});
test('chiusura sessione viene riflessa in tempo reale sul browser mobile',()=>{assert.match(rust,/values\[2\]=="state"/);assert.match(rust,/davWatchSession/);assert.match(rust,/davEndSession/);assert.match(rust,/Sessione terminata/);assert.match(rust,/window\.__davShareTransfers/);});

test('pagina mobile attende davvero Plus Jakarta Sans prima del rendering',()=>{assert.match(rust,/rel=\"preload\"/);assert.match(rust,/document\.fonts\.load\('400 16px/);assert.match(rust,/document\.fonts\.load\('800 16px/);assert.match(rust,/dav-font-loading/);assert.match(rust,/dav-font-ready/);assert.match(rust,/font-synthesis:none/);assert.match(rust,/-webkit-text-size-adjust:100%/);});

test('CSP HTTP mobile autorizza realmente Plus Jakarta Sans',()=>{assert.match(rust,/Content-Security-Policy\",\"default-src 'self';[^\"]*font-src https:\/\/fonts\.gstatic\.com/);});
test('pagina mobile verifica il font con FontFace prima del rendering',()=>{assert.match(rust,/new FontFace\('Plus Jakarta Sans'/);assert.match(rust,/document\.fonts\.add\(face\)/);assert.match(rust,/document\.fonts\.check\('400 16px/);assert.match(rust,/dataset\.davFont='jakarta'/);});


test('workflow GitHub pubblica release stabile con description bilingue',()=>{const workflow=fs.readFileSync('.github/workflows/release.yml','utf8');assert.match(workflow,/name: Release _davSHARE/);assert.match(workflow,/push:[\s\S]*tags:[\s\S]*'v\*'/);assert.match(workflow,/Verify release versions/);assert.match(workflow,/Read release description from tagged commit/);assert.match(workflow,/git log -1 --pretty=%b/);assert.match(workflow,/releaseBody:\s*\$\{\{ steps\.release_description\.outputs\.body \}\}/);assert.match(workflow,/tauri-apps\/tauri-action@v1/);assert.match(workflow,/--bundles nsis/);assert.match(workflow,/--target universal-apple-darwin --bundles dmg/);assert.match(workflow,/--bundles appimage,deb/);assert.match(workflow,/releaseDraft: false/);assert.match(workflow,/prerelease: false/);assert.match(workflow,/github\.ref_name/);assert.doesNotMatch(workflow,/Stable release of _davSHARE|generateReleaseNotes:\s*true/);});

test('Linux release workflow ignores unrelated Microsoft apt repository',()=>{const workflow=fs.readFileSync('.github/workflows/release.yml','utf8');assert.match(workflow,/packages\.microsoft\.com/);assert.match(workflow,/disabled-davshare/);assert.match(workflow,/Acquire::Retries=3/);assert.match(workflow,/--no-install-recommends/);});


test('metadata pacchetto _davstudios presenti',()=>{const pkg=JSON.parse(fs.readFileSync('package.json','utf8'));const tauri=JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json','utf8'));const cargo=fs.readFileSync('src-tauri/Cargo.toml','utf8');assert.equal(pkg.author,'_davstudios');assert.equal(pkg.license,'MIT');assert.equal(pkg.homepage,'https://davstudios.it');assert.equal(tauri.identifier,'studio.dav.share');assert.equal(tauri.bundle.category,'Productivity');assert.equal(tauri.bundle.publisher,'_davstudios');assert.equal(tauri.bundle.homepage,'https://davstudios.it');assert.equal(tauri.bundle.copyright,'© 2026 _davstudios');assert.equal(tauri.bundle.license,'MIT');assert.equal(tauri.bundle.licenseFile,'../LICENSE');assert.equal(tauri.bundle.linux.deb.section,'utils');assert.equal(tauri.bundle.linux.deb.priority,'optional');assert.match(cargo,/license = "MIT"/);assert.match(cargo,/homepage = "https:\/\/davstudios\.it"/);});

test('identifier storico resta invariato',()=>{const tauri=JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json','utf8'));assert.equal(tauri.identifier,'studio.dav.share');});

test('package metadata documenta lo standard release',()=>{const metadata=fs.readFileSync('PACKAGE-METADATA.md','utf8');assert.match(metadata,/Version: `26\.10\.1`/);assert.match(metadata,/Public release tag: `v26\.10\.1`/);assert.match(metadata,/Developer \/ Publisher: `_davstudios`/);assert.match(metadata,/Identifier: `studio\.dav\.share`/);assert.match(metadata,/Category: `Productivity`/);});
