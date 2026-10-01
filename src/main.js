import './styles.css';
import './motion.css';
import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { open } from '@tauri-apps/plugin-dialog';
import { openUrl } from '@tauri-apps/plugin-opener';
import { formatBytes,totalBytes,uniqueFiles,remainingSeconds,formatRemaining,transferRate,transferEta } from './share-engine.js';

const icons={
  send:'<svg viewBox="0 0 24 24"><path d="M22 2 11 13"/><path d="m22 2-7 20-4-9-9-4z"/></svg>',
  receive:'<svg viewBox="0 0 24 24"><path d="M12 3v13"/><path d="m7 11 5 5 5-5"/><path d="M5 21h14"/></svg>',
  activity:'<svg viewBox="0 0 24 24"><path d="M4 12h3l2-6 4 12 2-6h5"/></svg>',
  settings:'<svg class="nav-settings-gear" viewBox="0 0 24 24"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.09a2 2 0 0 1 1 1.74v.5a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.38a2 2 0 0 0-.73-2.73l-.15-.09a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>',
  plus:'<svg viewBox="0 0 24 24"><path d="M12 5v14M5 12h14"/></svg>',
  file:'<svg viewBox="0 0 24 24"><path d="M6 2h8l4 4v16H6z"/><path d="M14 2v5h5"/></svg>',
  trash:'<svg viewBox="0 0 24 24"><path d="M4 7h16M9 7V4h6v3M7 7l1 14h8l1-14"/></svg>',
  qr:'<svg viewBox="0 0 24 24"><rect x="3" y="3" width="6" height="6"/><rect x="15" y="3" width="6" height="6"/><rect x="3" y="15" width="6" height="6"/><path d="M14 14h3v3h-3zM18 14h3M21 17v4M14 19v2M17 21h2"/></svg>',
  copy:'<svg viewBox="0 0 24 24"><rect x="8" y="8" width="12" height="12" rx="2"/><path d="M16 8V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h3"/></svg>',
  stop:'<svg viewBox="0 0 24 24"><rect x="5" y="5" width="14" height="14" rx="2"/></svg>',
  folder:'<svg viewBox="0 0 24 24"><path d="M3 6h6l2 2h10v11H3z"/></svg>',
  shield:'<svg viewBox="0 0 24 24"><path d="M12 3 5 6v5c0 4.5 2.8 8 7 10 4.2-2 7-5.5 7-10V6z"/><path d="m9 12 2 2 4-4"/></svg>',
  wifi:'<svg viewBox="0 0 24 24"><path d="M5 12.5a10 10 0 0 1 14 0M8 16a6 6 0 0 1 8 0M11 19.5a2 2 0 0 1 2 0"/></svg>',
  globe:'<svg viewBox="0 0 390 390" aria-hidden="true"><path d="M195,0C87.305,0,0,87.304,0,195s87.305,195,195,195s195-87.304,195-195S302.695,0,195,0z M119.524,45.678c-3.493,4.838-6.838,10.033-10.007,15.6c-4.841,8.503-9.16,17.656-12.945,27.33c-8.064-2.22-16.089-4.713-24.064-7.483C85.91,66.718,101.813,54.667,119.524,45.678z M52.298,107.694c11.438,4.293,22.976,8.056,34.591,11.293c-4.78,18.934-7.744,39.182-8.745,60.087h-49.72C30.888,153.108,39.305,128.852,52.298,107.694z M52.298,282.306c-12.994-21.159-21.411-45.414-23.874-71.38h49.72c1.002,20.905,3.965,41.153,8.745,60.087C75.274,274.25,63.736,278.013,52.298,282.306z M72.508,308.876c7.975-2.77,16-5.265,24.063-7.483c3.786,9.674,8.105,18.827,12.946,27.33c3.168,5.566,6.514,10.762,10.007,15.6C101.813,335.333,85.91,323.283,72.508,308.876z M179.074,354.07c-20.393-7.648-38.458-29.593-51.05-59.894c16.931-3.125,33.977-5.059,51.05-5.8V354.07z M179.074,256.454c-20.448,0.818-40.862,3.221-61.117,7.191c-4.16-16.355-6.908-34.13-7.915-52.72h69.032V256.454z M179.074,179.074h-69.032c1.007-18.59,3.755-36.365,7.915-52.72c20.254,3.971,40.669,6.373,61.117,7.191V179.074z M179.074,101.623c-17.073-.741-34.118-2.675-51.05-5.8c12.592-30.301,30.657-52.245,51.05-59.894V101.623z M337.703,107.697c12.993,21.157,21.409,45.412,23.872,71.377h-49.72c-1.001-20.903-3.965-41.151-8.744-60.083C314.727,115.754,326.266,111.992,337.703,107.697z M317.495,81.128c-7.975,2.77-16,5.265-24.065,7.484c-3.786-9.676-8.105-18.831-12.947-27.335c-3.169-5.566-6.514-10.762-10.006-15.6C288.189,54.668,304.092,66.72,317.495,81.128z M210.926,35.93c20.393,7.648,38.459,29.595,51.051,59.898c-16.931,3.124-33.977,5.057-51.051,5.797V35.93z M210.926,133.547c20.45-.817,40.865-3.219,61.118-7.188c4.16,16.354,6.907,34.128,7.914,52.716h-69.032V133.547z M210.926,210.926h69.032c-1.007,18.588-3.754,36.362-7.914,52.716c-20.253-3.97-40.668-6.371-61.118-7.189V210.926z M210.926,354.07v-65.694c17.075.741,34.121,2.673,51.051,5.798C249.385,324.475,231.319,346.422,210.926,354.07z M270.477,344.322c3.493-4.838,6.838-10.033,10.006-15.6c4.842-8.504,9.161-17.659,12.947-27.334c8.064,2.22,16.089,4.714,24.065,7.484C304.092,323.28,288.189,335.332,270.477,344.322z M337.703,282.304c-11.437-4.296-22.976-8.058-34.591-11.296c4.779-18.932,7.742-39.179,8.744-60.082h49.72C359.112,236.891,350.696,261.146,337.703,282.304z"/></svg>',
  coffee:'<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m20.216 6.415-.132-.666c-.119-.598-.388-1.163-1.001-1.379-.197-.069-.42-.098-.57-.241-.152-.143-.196-.366-.231-.572-.065-.378-.125-.756-.192-1.133-.057-.325-.102-.69-.25-.987-.195-.4-.597-.634-.996-.788a5.723 5.723 0 0 0-.626-.194c-1-.263-2.05-.36-3.077-.416a25.834 25.834 0 0 0-3.7.062c-.915.083-1.88.184-2.75.5-.318.116-.646.256-.888.501-.297.302-.393.77-.177 1.146.154.267.415.456.692.58.36.162.737.284 1.123.366 1.075.238 2.189.331 3.287.37 1.218.05 2.437.01 3.65-.118.299-.033.598-.073.896-.119.352-.054.578-.513.474-.834-.124-.383-.457-.531-.834-.473-.466.074-.96.108-1.382.146-1.177.08-2.358.082-3.536.006a22.228 22.228 0 0 1-1.157-.107c-.086-.01-.18-.025-.258-.036-.243-.036-.484-.08-.724-.13-.111-.027-.111-.185 0-.212h.005c.277-.06.557-.108.838-.147h.002c.131-.009.263-.032.394-.048a25.076 25.076 0 0 1 3.426-.12c.674.019 1.347.067 2.017.144l.228.031c.267.04.533.088.798.145.392.085.895.113 1.07.542.055.137.08.288.111.431l.319 1.484a.237.237 0 0 1-.199.284h-.003c-.037.006-.075.01-.112.015a36.704 36.704 0 0 1-4.743.295 37.059 37.059 0 0 1-4.699-.304c-.14-.017-.293-.042-.417-.06-.326-.048-.649-.108-.973-.161-.393-.065-.768-.032-1.123.161-.29.16-.527.404-.675.701-.154.316-.199.66-.267 1-.069.34-.176.707-.135 1.056.087.753.613 1.365 1.37 1.502a39.69 39.69 0 0 0 11.343.376.483.483 0 0 1 .535.53l-.071.697-1.018 9.907c-.041.41-.047.832-.125 1.237-.122.637-.553 1.028-1.182 1.171-.577.131-1.165.2-1.756.205-.656.004-1.31-.025-1.966-.022-.699.004-1.556-.06-2.095-.58-.475-.458-.54-1.174-.605-1.793l-.731-7.013-.322-3.094c-.037-.351-.286-.695-.678-.678-.336.015-.718.3-.678.679l.228 2.185.949 9.112c.147 1.344 1.174 2.068 2.446 2.272.742.12 1.503.144 2.257.156.966.016 1.942.053 2.892-.122 1.408-.258 2.465-1.198 2.616-2.657.34-3.332.683-6.663 1.024-9.995l.215-2.087a.484.484 0 0 1 .39-.426c.402-.078.787-.212 1.074-.518.455-.488.546-1.124.385-1.766zm-1.478.772c-.145.137-.363.201-.578.233-2.416.359-4.866.54-7.308.46-1.748-.06-3.477-.254-5.207-.498-.17-.024-.353-.055-.47-.18-.22-.236-.111-.71-.054-.995.052-.26.152-.609.463-.646.484-.057 1.046.148 1.526.22.577.088 1.156.159 1.737.212 2.48.226 5.002.19 7.472-.14.45-.06.899-.13 1.345-.21.399-.072.84-.206 1.08.206.166.281.188.657.162.974a.544.544 0 0 1-.169.364zm-6.159 3.9c-.862.37-1.84.788-3.109.788a5.884 5.884 0 0 1-1.569-.217l.877 9.004c.065.78.717 1.38 1.5 1.38 0 0 1.243.065 1.658.065.447 0 1.786-.065 1.786-.065.783 0 1.434-.6 1.499-1.38l.94-9.95a3.996 3.996 0 0 0-1.322-.238c-.826 0-1.491.284-2.26.613z"/></svg>',
  reveal:'<svg viewBox="0 0 24 24"><path d="M4 5h6l2 2h8v12H4z"/><path d="m12 11 3 3-3 3M8 14h7"/></svg>',
  sun:'<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/></svg>',
  moon:'<svg viewBox="0 0 24 24"><path d="M20 15.5A8.5 8.5 0 0 1 8.5 4 8.5 8.5 0 1 0 20 15.5z"/></svg>',
  check:'<svg viewBox="0 0 24 24"><path d="m5 12 4 4 10-10"/></svg>'
};

const app=document.querySelector('#app');
const isTauri='__TAURI_INTERNALS__' in window;
const stored=JSON.parse(localStorage.getItem('davshare-settings')||'{}');
const storedHistory=JSON.parse(localStorage.getItem('davshare-history')||'[]');
const state={
  page:'send',version:'',files:[],runtime:null,diagnostics:null,session:null,status:null,busy:false,dragging:false,history:Array.isArray(storedHistory)?storedHistory.slice(0,50):[],statusShape:'',
  settings:{theme:stored.theme||'system',language:stored.language||'it',deviceName:stored.deviceName||'',receiveDir:stored.receiveDir||'',requireConfirmation:stored.requireConfirmation!==false,uploadLimitGb:String(stored.uploadLimitGb||'10'),networkIp:stored.networkIp||'auto'}
};

const t=(it,en)=>state.settings.language==='en'?en:it;
const esc=(value)=>String(value??'').replace(/[&<>'"]/g,(char)=>({'&':'&amp;','<':'&lt;','>':'&gt;',"'":'&#39;','"':'&quot;'})[char]);
const resolvedTheme=()=>state.settings.theme==='system'?(matchMedia('(prefers-color-scheme: dark)').matches?'dark':'light'):state.settings.theme;
function persist(){localStorage.setItem('davshare-settings',JSON.stringify(state.settings));}
function persistHistory(){localStorage.setItem('davshare-history',JSON.stringify(state.history.slice(0,50)));}
function applyTheme(){document.documentElement.dataset.theme=resolvedTheme();document.documentElement.lang=state.settings.language;}
function toast(message){const region=document.querySelector('#toast-region');if(!region)return;const node=document.createElement('div');node.className='toast';node.textContent=message;region.appendChild(node);setTimeout(()=>{node.classList.add('is-leaving');setTimeout(()=>node.remove(),180);},3600);}
function navButton(page,icon,label){return `<button class="nav-item ${state.page===page?'active':''}" data-page="${page}">${icon}<span>${label}</span></button>`;}
function runUiTransition(kind,change){document.documentElement.dataset.uiTransition=kind;if(document.startViewTransition){const transition=document.startViewTransition(change);transition.finished.finally(()=>delete document.documentElement.dataset.uiTransition);return;}change();setTimeout(()=>delete document.documentElement.dataset.uiTransition,360);}
function pageMeta(){
  if(state.page==='receive')return [t('Ricevi','Receive'),t('Ricevi file da Android, iPhone, iPad o qualsiasi browser sulla stessa rete.','Receive files from Android, iPhone, iPad or any browser on the same network.')];
  if(state.page==='activity')return [t('Attività','Activity'),t('Segui i trasferimenti in tempo reale e consulta la cronologia locale.','Track transfers in real time and review local history.')];
  if(state.page==='settings')return [t('Impostazioni','Settings'),t('Personalizza dispositivo, sicurezza, limiti e diagnostica della rete locale.','Customize device, security, limits and local-network diagnostics.')];
  return [t('Invia','Send'),t('Condividi file e cartelle dal computer con un QR temporaneo, senza cloud.','Share files and folders from your computer with a temporary QR code, without cloud storage.')];
}
function shell(content,motion='page'){
  const [title,subtitle]=pageMeta();
  app.innerHTML=`<div class="shell" data-motion-mode="${motion}"><aside class="sidebar"><div class="brand"><span>_dav</span>SHARE</div><nav>${navButton('send',icons.send,t('Invia','Send'))}${navButton('receive',icons.receive,t('Ricevi','Receive'))}${navButton('activity',icons.activity,t('Attività','Activity'))}${navButton('settings',icons.settings,t('Impostazioni','Settings'))}</nav><div class="sidebar-bottom"><button class="coffee-button" data-coffee>${icons.coffee}<span>${t('Comprami Un Caffè','Buy Me A Coffee')}</span></button><button class="icon-button theme-toggle" data-theme-toggle title="${t('Cambia tema','Change theme')}" aria-label="${t('Cambia tema','Change theme')}"><span class="theme-icon theme-icon-sun">${icons.sun}</span><span class="theme-icon theme-icon-moon">${icons.moon}</span></button></div></aside><main class="main"><header class="topbar"><div><div class="eyebrow">_DAVSHARE · V${esc(state.version)}</div><h1>${title}</h1><p class="page-subtitle">${subtitle}</p></div><div class="top-actions">${state.session?`<div class="session-pill"><span></span>${t('Sessione attiva','Session active')}</div>`:''}</div></header>${content}</main></div>${state.busy?`<div class="busy-indicator"><span class="busy-spinner"></span><span>${t('Preparazione sessione locale…','Preparing local session…')}</span></div>`:''}<div id="toast-region"></div>`;
  bindGlobal();
}
function bindGlobal(){
  document.querySelectorAll('[data-page]').forEach((node)=>node.addEventListener('click',()=>{state.page=node.dataset.page;render('page');}));
  document.querySelector('[data-coffee]')?.addEventListener('click',()=>openExternal('https://buymeacoffee.com/davstudios'));
  document.querySelector('[data-theme-toggle]')?.addEventListener('click',()=>{runUiTransition('theme',()=>{state.settings.theme=resolvedTheme()==='dark'?'light':'dark';persist();applyTheme();render('content');});});
  document.querySelectorAll('[data-stop-session]').forEach((node)=>node.addEventListener('click',stopSession));
  document.querySelectorAll('[data-copy-url]').forEach((node)=>node.addEventListener('click',copySessionUrl));
  document.querySelectorAll('[data-reveal]').forEach((node)=>node.addEventListener('click',()=>reveal(node.dataset.reveal)));
  document.querySelectorAll('[data-approve-client]').forEach((node)=>node.addEventListener('click',()=>decideClient(node.dataset.approveClient,true)));
  document.querySelectorAll('[data-reject-client]').forEach((node)=>node.addEventListener('click',()=>decideClient(node.dataset.rejectClient,false)));
}
async function ensureRuntime(force=false){
  if(state.runtime&&!force)return state.runtime;
  if(!isTauri)throw new Error(t('Apri _davSHARE come applicazione desktop.','Open _davSHARE as a desktop application.'));
  state.runtime=await invoke('runtime_info');
  if(!state.settings.deviceName)state.settings.deviceName=state.runtime.deviceName;
  if(!state.settings.receiveDir)state.settings.receiveDir=state.runtime.defaultReceiveDir;
  persist();
  try{state.diagnostics=await invoke('network_diagnostics');}catch{}
  return state.runtime;
}
function itemIcon(file){return file.kind==='folder'||file.kind==='archive'?icons.folder:icons.file;}
function itemType(file){if(file.kind==='folder')return t('Cartella · verrà preparata come ZIP','Folder · will be prepared as ZIP');if(file.kind==='archive')return t('Archivio ZIP temporaneo','Temporary ZIP archive');return t('File','File');}
function fileRow(file){return `<div class="share-file-row"><div class="file-symbol">${itemIcon(file)}</div><div class="file-copy"><strong>${esc(file.name)}</strong><span>${itemType(file)} · ${formatBytes(file.size)}</span></div><button class="icon-button compact" data-remove-file="${esc(file.path)}" title="${t('Rimuovi','Remove')}">×</button></div>`;}
function transferRow(item){
  const total=Number(item.total||0),done=Number(item.transferred||0),pct=total>0?Math.min(100,Math.round(done/total*100)):0,rate=transferRate(done,item.startedAt),eta=transferEta(total,done,rate);
  return `<div class="live-transfer" data-transfer-id="${esc(item.id)}"><div class="transfer-head"><div><strong>${esc(item.name)}</strong><span>${item.direction==='send'?t('Invio al dispositivo','Sending to device'):t('Ricezione dal dispositivo','Receiving from device')} · ${esc(item.clientIp||'')}</span></div><b data-transfer-label="${esc(item.id)}">${total?`${pct}% · ${formatBytes(done)} / ${formatBytes(total)} · ${formatBytes(rate)}/s${eta?` · ${formatRemaining(eta)}`:''}`:`${formatBytes(done)} · ${formatBytes(rate)}/s`}</b></div><div class="transfer-progress"><i data-transfer-progress="${esc(item.id)}" style="width:${pct}%"></i></div></div>`;
}
function approvalBlock(){
  const pending=state.status?.pendingClients||[];
  if(!pending.length)return '';
  return `<div class="approval-stack"><div class="approval-title">${icons.shield}<div><strong>${t('Richiesta di connessione','Connection request')}</strong><span>${t('Approva soltanto i dispositivi che riconosci.','Approve only devices you recognize.')}</span></div></div>${pending.map((client)=>`<div class="approval-row"><div><strong>${esc(client.ip)}</strong><small>${t('vuole accedere alla sessione','wants to access this session')}</small></div><div><button class="button secondary small" data-reject-client="${esc(client.ip)}">${t('Rifiuta','Reject')}</button><button class="button primary small" data-approve-client="${esc(client.ip)}">${t('Accetta','Approve')}</button></div></div>`).join('')}</div>`;
}
function qrSessionCard(mode){
  if(!state.session||state.session.mode!==mode)return '';
  const status=state.status||{};
  const seconds=remainingSeconds(state.session.expiresAt);
  const counter=mode==='send'?`${Number(status.downloads||0)} ${t('download','downloads')}`:`${Number(status.uploads||0)} ${t('upload','uploads')}`;
  const transfers=status.activeTransfers||[];
  return `<section class="panel qr-panel"><div class="panel-head"><div><h2>${t('Scansiona il QR','Scan the QR')}</h2><span>${t('Aprilo con la fotocamera del telefono.','Open it with your phone camera.')}</span></div><div class="live-dot"></div></div><div class="qr-wrap">${state.session.qrSvg}</div><div class="session-url"><span>${esc(state.session.url)}</span><button class="icon-button compact" data-copy-url>${icons.copy}</button></div><div class="session-meta"><div><span>${t('Scadenza','Expires')}</span><strong data-session-time>${formatRemaining(seconds)}</strong></div><div><span>${t('Completati','Completed')}</span><strong data-session-count>${counter}</strong></div></div>${approvalBlock()}${transfers.length?`<div class="live-transfer-stack">${transfers.map(transferRow).join('')}</div>`:''}<div class="qr-actions"><button class="button secondary" data-copy-url>${icons.copy}${t('Copia link','Copy link')}</button><button class="button danger" data-stop-session>${icons.stop}${t('Chiudi sessione','Stop session')}</button></div></section>`;
}
function networkNote(){
  const ip=state.session?.url?.match(/\/\/(\d+\.\d+\.\d+\.\d+)/)?.[1]||state.runtime?.localIp;
  return `<div class="panel privacy-banner">${icons.wifi}<div><strong>${t('Stessa rete locale','Same local network')}${ip?` · ${esc(ip)}`:''}</strong><span>${t('Computer e telefono devono essere sulla stessa Wi-Fi/LAN. _davSHARE ascolta sulle interfacce locali e il QR usa l’IP LAN selezionato. Se Safari non raggiunge la pagina, controlla anche che il firewall consenta _davSHARE sulle reti private.','Computer and phone must be on the same Wi-Fi/LAN. _davSHARE listens on local interfaces and the QR uses the selected LAN IP. If Safari cannot reach the page, also make sure the firewall allows _davSHARE on private networks.')}</span></div></div>`;
}
function sendPage(motion='page'){
  const total=totalBytes(state.files),active=state.session?.mode==='send';
  shell(`<section class="share-page send-page"><div class="share-grid"><section class="panel files-panel"><div class="panel-head"><div><h2>${t('File e cartelle da condividere','Files and folders to share')}</h2><span>${state.files.length?`${state.files.length} · ${formatBytes(total)}`:t('Seleziona file o cartelle','Select files or folders')}</span></div>${state.files.length?`<button class="button ghost small" data-clear-files>${icons.trash}${t('Svuota','Clear')}</button>`:''}</div>${state.files.length?`<div class="share-file-list">${state.files.map(fileRow).join('')}</div><div class="files-actions wrap-actions"><button class="button secondary" data-add-files>${icons.plus}${t('Aggiungi file','Add files')}</button><button class="button secondary" data-add-folder>${icons.folder}${t('Aggiungi cartella','Add folder')}</button><button class="button primary" data-start-send ${state.busy?'disabled':''}>${icons.qr}${active?t('Rigenera QR','Regenerate QR'):t('Genera QR','Generate QR')}</button></div>`:`<div class="share-drop-zone" data-drop-zone><div class="drop-mark">${icons.send}</div><h2>${t('Scegli cosa inviare','Choose what to send')}</h2><p>${t('Trascina qui file o cartelle. Le cartelle vengono preparate automaticamente come ZIP temporanei.','Drop files or folders here. Folders are automatically prepared as temporary ZIP archives.')}</p><div class="drop-actions"><button class="button primary" data-add-files>${icons.plus}${t('Seleziona file','Select files')}</button><button class="button secondary" data-add-folder>${icons.folder}${t('Seleziona cartella','Select folder')}</button></div><small>${t('Il file ZIP temporaneo viene eliminato quando chiudi o rigeneri la sessione.','The temporary ZIP is deleted when you stop or regenerate the session.')}</small></div>`}</section>${active?qrSessionCard('send'):`<section class="panel ready-panel"><div class="ready-mark">${icons.qr}</div><h2>${t('Un QR, nessuna app sul telefono','One QR, no phone app')}</h2><p>${t('Android, iPhone e iPad aprono una pagina temporanea nel browser. I file restano nella LAN e più dispositivi possono trasferire contemporaneamente.','Android, iPhone and iPad open a temporary browser page. Files stay on the LAN and multiple devices can transfer concurrently.')}</p><div class="feature-lines"><span>${icons.shield}${t('Conferma dispositivo opzionale','Optional device approval')}</span><span>${icons.wifi}${t('Server LAN dedicato','Dedicated LAN server')}</span><span>${icons.qr}${t('Sessione di 10 minuti','10-minute session')}</span></div></section>`}</div>${networkNote()}</section>`,motion);
  document.querySelectorAll('[data-add-files]').forEach((node)=>node.addEventListener('click',chooseFiles));
  document.querySelectorAll('[data-add-folder]').forEach((node)=>node.addEventListener('click',chooseFolder));
  document.querySelector('[data-clear-files]')?.addEventListener('click',async()=>{state.files=[];if(state.session?.mode==='send')await stopSession(false);render('content');});
  document.querySelectorAll('[data-remove-file]').forEach((node)=>node.addEventListener('click',async()=>{state.files=state.files.filter((file)=>file.path!==node.dataset.removeFile);if(state.session?.mode==='send')await stopSession(false);render('content');}));
  document.querySelector('[data-start-send]')?.addEventListener('click',()=>startSession('send'));
}
function receivePage(motion='page'){
  const active=state.session?.mode==='receive';
  const dir=state.settings.receiveDir||state.runtime?.defaultReceiveDir||t('Cartella Download / _davSHARE','Downloads / _davSHARE folder');
  const limit=Number(state.settings.uploadLimitGb)||10;
  shell(`<section class="share-page receive-page"><div class="share-grid"><section class="panel receive-panel"><div class="receive-hero"><div class="ready-mark">${icons.receive}</div><div><h2>${t('Ricevi dal telefono','Receive from your phone')}</h2><p>${t('Genera un QR e dal telefono seleziona foto, video o documenti. Il progresso viene mostrato in tempo reale sul computer.','Generate a QR and select photos, videos or documents from your phone. Progress is shown live on the computer.')}</p></div></div><div class="receive-destination"><span>${t('Cartella di destinazione','Destination folder')}</span><strong>${esc(dir)}</strong><button class="button secondary small" data-change-folder>${t('Cambia','Change')}</button></div><div class="receive-safety"><span>${icons.shield}${state.settings.requireConfirmation?t('Conferma dispositivo attiva','Device approval enabled'):t('Conferma dispositivo disattivata','Device approval disabled')}</span><span>${icons.file}${t(`Limite ${limit} GB per file`,`Limit ${limit} GB per file`)}</span></div><button class="button primary full" data-start-receive ${state.busy?'disabled':''}>${icons.qr}${active?t('Rigenera QR di ricezione','Regenerate receive QR'):t('Genera QR di ricezione','Generate receive QR')}</button></section>${active?qrSessionCard('receive'):`<section class="panel ready-panel"><div class="ready-mark">${icons.wifi}</div><h2>${t('Il browser diventa il mittente','The browser becomes the sender')}</h2><p>${t('Non servono APK o IPA. Il telefono carica direttamente verso il PC sulla stessa rete locale.','No APK or IPA is required. The phone uploads directly to the PC on the same local network.')}</p><div class="feature-lines"><span>${icons.shield}${t('Spazio libero controllato','Free space checked')}</span><span>${icons.folder}${t('Nomi file protetti','Protected filenames')}</span><span>${icons.stop}${t('Stop interrompe anche gli upload','Stop interrupts active uploads')}</span></div></section>`}</div>${networkNote()}</section>`,motion);
  document.querySelector('[data-change-folder]')?.addEventListener('click',chooseReceiveFolder);
  document.querySelector('[data-start-receive]')?.addEventListener('click',()=>startSession('receive'));
}
function historyRow(item){
  const direction=item.direction==='send'?t('Inviato','Sent'):t('Ricevuto','Received');
  const status=item.status==='completed'?t('Completato','Completed'):item.status==='rejected'?t('Rifiutato','Rejected'):t('Interrotto','Interrupted');
  const when=item.completedAt?new Date(Number(item.completedAt)*1000).toLocaleString():'';
  return `<div class="history-row"><div class="history-symbol">${item.direction==='send'?icons.send:icons.receive}</div><div><strong>${esc(item.name)}</strong><span>${direction} · ${esc(item.clientIp||'')} · ${formatBytes(item.size||0)}</span></div><div><b class="history-status ${esc(item.status||'')}">${status}</b><small>${esc(when)}</small></div></div>`;
}
function activityPage(motion='page'){
  const status=state.status||{},transfers=status.activeTransfers||[],history=state.history||[],received=status.received||[];
  shell(`<section class="share-page activity-page"><section class="share-stats"><div class="panel stat-card"><span>${t('Dati inviati','Data sent')}</span><strong data-bytes-sent>${formatBytes(status.bytesSent||0)}</strong><small>${Number(status.downloads||0)} ${t('download completati','completed downloads')}</small></div><div class="panel stat-card"><span>${t('Dati ricevuti','Data received')}</span><strong data-bytes-received>${formatBytes(status.bytesReceived||0)}</strong><small>${Number(status.uploads||0)} ${t('upload completati','completed uploads')}</small></div><div class="panel stat-card"><span>${t('Trasferimenti attivi','Active transfers')}</span><strong>${transfers.length}</strong><small>${state.session?t('sessione locale attiva','local session active'):t('nessuna sessione','no session')}</small></div></section><section class="activity-grid"><section class="panel activity-panel"><div class="panel-head"><div><h2>${t('In tempo reale','Live')}</h2><span>${transfers.length?t('Progresso in byte aggiornato automaticamente.','Byte-level progress updates automatically.'):t('Nessun trasferimento in corso.','No transfer in progress.')}</span></div>${icons.activity}</div>${transfers.length?`<div class="live-transfer-stack">${transfers.map(transferRow).join('')}</div>`:`<div class="activity-empty compact-empty"><div class="ready-mark">${icons.activity}</div><h2>${t('In attesa','Waiting')}</h2><p>${t('Scansiona il QR da un altro dispositivo per iniziare.','Scan the QR from another device to begin.')}</p></div>`}${approvalBlock()}</section><section class="panel history-panel"><div class="panel-head"><div><h2>${t('Cronologia locale','Local history')}</h2><span>${t('Ultimi 50 trasferimenti, solo metadati.','Last 50 transfers, metadata only.')}</span></div><button class="button ghost small" data-clear-history ${history.length?'':'disabled'}>${icons.trash}${t('Svuota','Clear')}</button></div>${history.length?`<div class="history-list">${history.map(historyRow).join('')}</div>`:`<div class="list-empty compact">${t('Nessun trasferimento completato o interrotto.','No completed or interrupted transfers.')}</div>`}${received.length?`<div class="received-list"><h3>${t('File ricevuti in questa sessione','Files received in this session')}</h3>${received.map((item)=>`<button class="received-row" data-reveal="${esc(item.path)}"><span>${icons.file}</span><div><strong>${esc(item.name)}</strong><small>${formatBytes(item.size)}</small></div>${icons.reveal}</button>`).join('')}</div>`:''}</section></section></section>`,motion);
  document.querySelector('[data-clear-history]')?.addEventListener('click',()=>{state.history=[];persistHistory();render('content');});
}
function davSelect(key,value,items){const selected=items.find(([item])=>item===value)?.[1]||value;return `<div class="dav-select" data-select="${key}"><button class="dav-select-trigger"><span>${esc(selected)}</span><span>⌄</span></button><div class="dav-select-menu">${items.map(([item,label])=>`<button class="dav-select-option ${item===value?'is-selected':''}" data-value="${item}"><span>${esc(label)}</span>${item===value?icons.check:''}</button>`).join('')}</div></div>`;}
function diagnosticsCard(){
  const d=state.diagnostics,r=state.runtime;
  if(!d&&!r)return `<div class="network-diagnostic"><span>${t('Diagnostica non ancora eseguita','Diagnostics not run yet')}</span><button class="button secondary small" data-network-check>${t('Verifica rete','Check network')}</button></div>`;
  const ips=d?.interfaceIps||r?.interfaceIps||[];
  return `<div class="network-diagnostic"><div class="diagnostic-grid"><div><span>${t('IP server','Server IP')}</span><strong>${esc(d?.boundIp||r?.localIp||'—')}</strong></div><div><span>${t('Porta','Port')}</span><strong>${esc(d?.port||r?.port||'—')}</strong></div><div><span>${t('Self-test','Self-test')}</span><strong class="${d?.selfTest?'diag-ok':'diag-warn'}">${d?.selfTest?t('Raggiungibile','Reachable'):t('Da verificare','Check required')}</strong></div><div><span>${t('Interfacce LAN','LAN interfaces')}</span><strong>${esc(ips.join(', ')||'—')}</strong></div></div>${d?.networkChanged?`<p class="diagnostic-warning">${t('La rete è cambiata dopo l’avvio. Riavvia _davSHARE per associare il server alla nuova interfaccia.','The network changed after startup. Restart _davSHARE to bind the server to the new interface.')}</p>`:''}<button class="button secondary small" data-network-check>${icons.wifi}${t('Ricontrolla','Check again')}</button></div>`;
}
function settingsPage(motion='page'){
  const receive=state.settings.receiveDir||state.runtime?.defaultReceiveDir||t('Automatico al primo avvio','Automatic on first use');
  const networkItems=[['auto',t('Automatico','Automatic')],...((state.runtime?.interfaceIps||[]).map((ip)=>[ip,ip]))];
  shell(`<section class="settings-grid share-settings-page"><div class="panel settings-card"><h2>${t('Generali','General')}</h2><div class="setting-control"><span>${t('Tema','Theme')}</span>${davSelect('theme',state.settings.theme,[['system',t('Sistema','System')],['light',t('Chiaro','Light')],['dark',t('Scuro','Dark')]])}</div><div class="setting-control"><span>${t('Lingua','Language')}</span>${davSelect('language',state.settings.language,[['it','Italiano'],['en','English']])}</div><div class="setting-control"><span>${t('Rete locale','Local network')}</span>${davSelect('networkIp',state.settings.networkIp,networkItems)}</div><div class="setting-control"><span>${t('Limite upload per file','Per-file upload limit')}</span>${davSelect('uploadLimitGb',state.settings.uploadLimitGb,[['1','1 GB'],['5','5 GB'],['10','10 GB'],['25','25 GB'],['100','100 GB']])}</div><label class="text-setting"><span>${t('Nome dispositivo','Device name')}</span><input data-device-name value="${esc(state.settings.deviceName)}" placeholder="${t('Questo computer','This computer')}"></label><label class="setting-check share-check"><input type="checkbox" data-confirm-client ${state.settings.requireConfirmation?'checked':''}><span><strong>${t('Richiedi conferma dispositivo','Require device approval')}</strong><small>${t('Prima di mostrare upload/download, il PC deve accettare l’indirizzo del dispositivo.','Before showing upload/download, the PC must approve the device address.')}</small></span></label><div class="folder-setting"><span>${t('Cartella di ricezione','Receive folder')}</span><div><strong>${esc(receive)}</strong><button class="button secondary small" data-folder>${t('Cambia','Change')}</button></div></div></div><div class="panel about-card"><div class="brand big"><span>_dav</span>SHARE</div><p>${t('Condivisione locale tra desktop e dispositivi mobili tramite QR temporanei.','Local sharing between desktop and mobile devices through temporary QR codes.')}</p><div class="about-links"><button class="website-button" data-site>${icons.globe}<span>davstudios.it</span></button><button class="coffee-button wide" data-coffee-about>${icons.coffee}<span>${t('Comprami Un Caffè','Buy Me A Coffee')}</span></button></div><div class="version">v${esc(state.version)}</div></div><div class="panel capability-card safety-principles"><h2>${t('Rete e sicurezza','Network & security')}</h2>${diagnosticsCard()}<div class="capability-list"><div><strong>${t('Nessun Internet richiesto','No Internet required')}</strong><span>${t('L’IP viene ricavato dalle interfacce LAN locali, non tramite servizi esterni.','The IP is discovered from local LAN interfaces, not through external services.')}</span></div><div><strong>${t('Ascolto multi-interfaccia','Multi-interface listening')}</strong><span>${t('Il server ascolta sulle interfacce locali, mentre token e controllo LAN limitano l’accesso alla sessione.','The server listens on local interfaces while the token and LAN checks restrict access to the session.')}</span></div><div><strong>${t('Stop reale','Real cancellation')}</strong><span>${t('Chiudere la sessione invalida il token e interrompe i flussi ancora attivi.','Stopping the session invalidates the token and interrupts active streams.')}</span></div><div><strong>${t('Percorsi protetti','Protected paths')}</strong><span>${t('Nomi riservati e caratteri non validi vengono normalizzati e lo spazio libero viene verificato prima della ricezione.','Reserved names and invalid characters are normalized and free space is checked before receiving.')}</span></div></div></div></section>`,motion);
  bindSelects();
  document.querySelector('[data-device-name]')?.addEventListener('change',(event)=>{state.settings.deviceName=event.target.value.trim();persist();});
  document.querySelector('[data-confirm-client]')?.addEventListener('change',(event)=>{state.settings.requireConfirmation=event.target.checked;persist();if(state.session)stopSession();});
  document.querySelector('[data-folder]')?.addEventListener('click',chooseReceiveFolder);
  document.querySelectorAll('[data-network-check]').forEach((node)=>node.addEventListener('click',refreshDiagnostics));
  document.querySelector('[data-coffee-about]')?.addEventListener('click',()=>openExternal('https://buymeacoffee.com/davstudios'));
  document.querySelector('[data-site]')?.addEventListener('click',()=>openExternal(state.settings.language==='en'?'https://www.davstudios.it/en':'https://www.davstudios.it'));
}
function bindSelects(){
  document.querySelectorAll('.dav-select-trigger').forEach((node)=>node.addEventListener('click',(event)=>{event.stopPropagation();const root=node.closest('.dav-select');document.querySelectorAll('.dav-select.is-open').forEach((other)=>{if(other!==root)other.classList.remove('is-open');});root.classList.toggle('is-open');}));
  document.querySelectorAll('.dav-select-option').forEach((node)=>node.addEventListener('click',()=>{const root=node.closest('.dav-select');const key=root.dataset.select,value=node.dataset.value;runUiTransition(key==='theme'?'theme':'content',()=>{state.settings[key]=value;persist();applyTheme();if(state.session&&(key==='language'||key==='uploadLimitGb'||key==='networkIp'))stopSession(false);render('content');});}));
}
async function chooseFiles(){
  if(!isTauri)return toast(t('Apri l’app desktop per selezionare file.','Open the desktop app to select files.'));
  const selected=await open({multiple:true,directory:false});
  if(selected)await addPaths(Array.isArray(selected)?selected:[selected]);
}
async function chooseFolder(){
  if(!isTauri)return;
  const selected=await open({directory:true,multiple:false});
  if(selected)await addPaths([selected]);
}
async function addPaths(paths){
  try{const inspected=await invoke('inspect_share_files',{paths});state.files=uniqueFiles([...state.files,...inspected]);if(state.session?.mode==='send')await stopSession(false);render('content');}catch(error){toast(String(error));}
}
async function chooseReceiveFolder(){
  if(!isTauri)return;
  const selected=await open({directory:true,multiple:false});
  if(selected){state.settings.receiveDir=selected;persist();if(state.session?.mode==='receive')await stopSession(false);render('content');}
}
async function startSession(mode){
  if(state.busy)return;
  state.busy=true;render('content');
  try{
    await ensureRuntime();
    const maxUploadBytes=Math.round((Number(state.settings.uploadLimitGb)||10)*1024*1024*1024);
    const session=await invoke('start_share_session',{mode,paths:mode==='send'?state.files.map((file)=>file.path):[],receiveDir:state.settings.receiveDir,deviceName:state.settings.deviceName,language:state.settings.language,requireConfirmation:state.settings.requireConfirmation,maxUploadBytes,networkIp:state.settings.networkIp});
    state.session=session;state.status={active:true,mode,downloads:0,uploads:0,bytesSent:0,bytesReceived:0,received:[],activeTransfers:[],pendingClients:[],history:state.history};state.statusShape='';
    toast(mode==='send'?t('QR di condivisione pronto.','Share QR ready.'):t('QR di ricezione pronto.','Receive QR ready.'));
  }catch(error){toast(String(error));}
  finally{state.busy=false;render('content');}
}
async function stopSession(redraw=true){
  try{if(isTauri)await invoke('stop_share_session');}catch(error){toast(String(error));}
  state.session=null;state.status=null;state.statusShape='';
  if(redraw)render('content');
}
async function decideClient(ip,approve){
  try{await invoke(approve?'approve_client':'reject_client',{ip});toast(approve?t('Dispositivo autorizzato.','Device approved.'):t('Dispositivo rifiutato.','Device rejected.'));await pollStatus(true);}catch(error){toast(String(error));}
}
async function copySessionUrl(){if(!state.session)return;try{await navigator.clipboard.writeText(state.session.url);toast(t('Link copiato.','Link copied.'));}catch{toast(state.session.url);}}
async function reveal(path){if(!isTauri)return;try{await invoke('reveal_path',{path});}catch(error){toast(String(error));}}
async function openExternal(url){try{if(isTauri)await openUrl(url);else window.open(url,'_blank','noopener,noreferrer');}catch{window.open(url,'_blank','noopener,noreferrer');}}
async function refreshDiagnostics(){
  if(!isTauri)return;
  try{await ensureRuntime();state.diagnostics=await invoke('network_diagnostics');render('content');toast(state.diagnostics.selfTest?t('Server locale raggiungibile.','Local server reachable.'):t('Il self-test della rete non è riuscito.','Network self-test failed.'));}catch(error){toast(String(error));}
}
function syncHistory(items){
  if(!Array.isArray(items))return;
  const byId=new Map(state.history.map((item)=>[item.id,item]));
  for(const item of items)byId.set(item.id,item);
  state.history=[...byId.values()].sort((a,b)=>Number(b.completedAt||0)-Number(a.completedAt||0)).slice(0,50);
  persistHistory();
}
function updateLiveUi(){
  if(!state.session)return;
  const seconds=remainingSeconds(state.session.expiresAt);
  document.querySelector('[data-session-time]')?.replaceChildren(document.createTextNode(formatRemaining(seconds)));
  const count=state.session.mode==='send'?`${Number(state.status?.downloads||0)} ${t('download','downloads')}`:`${Number(state.status?.uploads||0)} ${t('upload','uploads')}`;
  document.querySelector('[data-session-count]')?.replaceChildren(document.createTextNode(count));
  for(const item of state.status?.activeTransfers||[]){const total=Number(item.total||0),done=Number(item.transferred||0),pct=total>0?Math.min(100,Math.round(done/total*100)):0,rate=transferRate(done,item.startedAt),eta=transferEta(total,done,rate);const bar=document.querySelector(`[data-transfer-progress="${CSS.escape(String(item.id))}"]`);if(bar)bar.style.width=`${pct}%`;const label=document.querySelector(`[data-transfer-label="${CSS.escape(String(item.id))}"]`);if(label)label.textContent=total?`${pct}% · ${formatBytes(done)} / ${formatBytes(total)} · ${formatBytes(rate)}/s${eta?` · ${formatRemaining(eta)}`:''}`:`${formatBytes(done)} · ${formatBytes(rate)}/s`; }
  document.querySelector('[data-bytes-sent]')?.replaceChildren(document.createTextNode(formatBytes(state.status?.bytesSent||0)));
  document.querySelector('[data-bytes-received]')?.replaceChildren(document.createTextNode(formatBytes(state.status?.bytesReceived||0)));
  if(seconds===0){state.session=null;state.status=null;state.statusShape='';render('content');toast(t('Sessione scaduta. Genera un nuovo QR.','Session expired. Generate a new QR.'));}
}
async function pollStatus(forceRender=false){
  if(!isTauri||!state.session)return updateLiveUi();
  try{
    const status=await invoke('share_status');syncHistory(status.history);state.status=status;
    if(!status.active){state.session=null;state.status=null;state.statusShape='';render('content');return;}
    const shape=JSON.stringify({transfers:(status.activeTransfers||[]).map((item)=>item.id),pending:(status.pendingClients||[]).map((item)=>item.ip),history:(status.history||[]).map((item)=>item.id)});
    const changed=shape!==state.statusShape;state.statusShape=shape;
    if(forceRender||changed&&(state.page==='activity'||state.page==='send'||state.page==='receive'))render('content');else updateLiveUi();
  }catch{}
}
function render(motion='page'){applyTheme();if(state.page==='receive')receivePage(motion);else if(state.page==='activity')activityPage(motion);else if(state.page==='settings')settingsPage(motion);else sendPage(motion);}
async function setupDragDrop(){
  if(!isTauri)return;
  try{await getCurrentWebview().onDragDropEvent((event)=>{if(event.payload.type==='over'){state.dragging=true;document.querySelector('[data-drop-zone]')?.classList.add('drag-over');}else if(event.payload.type==='drop'){state.dragging=false;document.querySelector('[data-drop-zone]')?.classList.remove('drag-over');if(state.page==='send')addPaths(event.payload.paths);}else{state.dragging=false;document.querySelector('[data-drop-zone]')?.classList.remove('drag-over');}});}catch{}
}
async function init(){
  applyTheme();document.addEventListener('click',()=>document.querySelectorAll('.dav-select.is-open').forEach((node)=>node.classList.remove('is-open')));
  if(isTauri){try{state.version=await getVersion();await ensureRuntime();}catch{}}
  render('startup');await setupDragDrop();setInterval(()=>pollStatus(false),800);
  matchMedia('(prefers-color-scheme: dark)').addEventListener?.('change',()=>{if(state.settings.theme==='system')render('content');});
}
init();
