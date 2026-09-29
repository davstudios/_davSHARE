use qrcode::{render::svg,QrCode};
use rand::{distributions::Alphanumeric,Rng};
use serde::Serialize;
use std::collections::{HashMap,HashSet};
use std::env;
use std::fs::{self,File};
use std::io::{self,Read,Write};
use std::net::{IpAddr,Ipv4Addr,TcpStream,UdpSocket};
use std::path::{Path,PathBuf};
use std::process::Command;
use std::sync::{Arc,Mutex};
use std::thread;
use std::time::{Duration,SystemTime,UNIX_EPOCH};
use tauri::Manager;
use tiny_http::{Header,Method,Request,Response,Server,StatusCode};
use zip::{write::FileOptions,CompressionMethod,ZipWriter};

const DEFAULT_PORT:u16=47821;
const SESSION_TTL_SECONDS:u64=600;
const DEFAULT_MAX_UPLOAD_BYTES:u64=10*1024*1024*1024;
const MIN_FREE_RESERVE_BYTES:u64=128*1024*1024;
const HISTORY_LIMIT:usize=50;

#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
struct ShareFile{
    path:String,
    name:String,
    size:u64,
    kind:String,
}

#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
struct ReceivedFile{
    name:String,
    path:String,
    size:u64,
}

#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
struct TransferProgress{
    id:String,
    name:String,
    direction:String,
    client_ip:String,
    transferred:u64,
    total:u64,
    started_at:u64,
}

#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
struct HistoryEntry{
    id:String,
    name:String,
    direction:String,
    client_ip:String,
    size:u64,
    completed_at:u64,
    status:String,
}

#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
struct PendingClient{
    ip:String,
    requested_at:u64,
}

#[derive(Clone)]
struct Session{
    token:String,
    mode:String,
    device_name:String,
    language:String,
    files:Vec<ShareFile>,
    receive_dir:PathBuf,
    expires_at:u64,
    downloads:u64,
    uploads:u64,
    bytes_sent:u64,
    bytes_received:u64,
    received:Vec<ReceivedFile>,
    last_activity:u64,
    local_ip:Ipv4Addr,
    require_confirmation:bool,
    max_upload_bytes:u64,
    pending_clients:HashMap<String,u64>,
    approved_clients:HashSet<String>,
    denied_clients:HashSet<String>,
    transfers:HashMap<String,TransferProgress>,
    temp_paths:Vec<PathBuf>,
}

#[derive(Default)]
struct SharedState{
    session:Option<Session>,
    history:Vec<HistoryEntry>,
}

struct Runtime{
    shared:Arc<Mutex<SharedState>>,
    server_binding:Mutex<Option<(Ipv4Addr,u16)>>,
}

impl Default for Runtime{
    fn default()->Self{
        Self{shared:Arc::new(Mutex::new(SharedState::default())),server_binding:Mutex::new(None)}
    }
}

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
struct RuntimeInfo{
    local_ip:String,
    port:u16,
    interface_ips:Vec<String>,
    device_name:String,
    default_receive_dir:String,
}

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
struct NetworkDiagnostics{
    bound_ip:String,
    port:u16,
    interface_ips:Vec<String>,
    self_test:bool,
    network_changed:bool,
}

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
struct SessionView{
    active:bool,
    mode:String,
    url:String,
    qr_svg:String,
    token:String,
    expires_at:u64,
    files:Vec<ShareFile>,
    receive_dir:String,
}

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
struct ShareStatus{
    active:bool,
    mode:String,
    expires_at:u64,
    downloads:u64,
    uploads:u64,
    bytes_sent:u64,
    bytes_received:u64,
    received:Vec<ReceivedFile>,
    last_activity:u64,
    active_transfers:Vec<TransferProgress>,
    pending_clients:Vec<PendingClient>,
    history:Vec<HistoryEntry>,
}

fn now()->u64{
    SystemTime::now().duration_since(UNIX_EPOCH).map(|value|value.as_secs()).unwrap_or(0)
}

fn parse_ipv4_tokens(text:&str)->Vec<Ipv4Addr>{
    let mut result=Vec::new();
    for token in text.split(|ch:char|!(ch.is_ascii_digit()||ch=='.')){
        if token.matches('.').count()!=3{continue;}
        if let Ok(ip)=token.parse::<Ipv4Addr>(){
            if ip.is_private()&&!ip.is_loopback()&&!ip.is_link_local()&&!result.contains(&ip){result.push(ip);}
        }
    }
    result
}

fn command_output(program:&str,args:&[&str])->String{
    Command::new(program).args(args).output().ok().map(|output|String::from_utf8_lossy(&output.stdout).into_owned()).unwrap_or_default()
}

fn local_ipv4_candidates()->Vec<Ipv4Addr>{
    let mut result=Vec::new();
    #[cfg(target_os="windows")]
    {
        let physical=command_output("powershell",&["-NoProfile","-Command","Get-NetIPConfiguration | Where-Object { $_.IPv4DefaultGateway -ne $null -and $_.NetAdapter.Status -eq 'Up' -and $_.NetAdapter.HardwareInterface -eq $true } | Sort-Object { $_.NetIPv4Interface.InterfaceMetric } | ForEach-Object { $_.IPv4Address.IPAddress }"]);
        result.extend(parse_ipv4_tokens(&physical));
        let routed=command_output("powershell",&["-NoProfile","-Command","Get-NetIPConfiguration | Where-Object { $_.IPv4DefaultGateway -ne $null -and $_.NetAdapter.Status -eq 'Up' } | Sort-Object { $_.NetIPv4Interface.InterfaceMetric } | ForEach-Object { $_.IPv4Address.IPAddress }"]);
        for ip in parse_ipv4_tokens(&routed){if !result.contains(&ip){result.push(ip);}}
        if result.is_empty(){result.extend(parse_ipv4_tokens(&command_output("ipconfig",&["/all"])));}
    }
    #[cfg(target_os="macos")]
    {
        result.extend(parse_ipv4_tokens(&command_output("ifconfig",&[])));
    }
    #[cfg(all(unix,not(target_os="macos")))]
    {
        result.extend(parse_ipv4_tokens(&command_output("hostname",&["-I"])));
        result.extend(parse_ipv4_tokens(&command_output("ip",&["-4","addr"])));
    }
    for probe in ["192.168.0.1:9","10.0.0.1:9","172.16.0.1:9"]{
        if let Ok(socket)=UdpSocket::bind("0.0.0.0:0"){
            if socket.connect(probe).is_ok(){
                if let Ok(address)=socket.local_addr(){
                    if let IpAddr::V4(ip)=address.ip(){if ip.is_private()&&!ip.is_loopback()&&!result.contains(&ip){result.push(ip);}}
                }
            }
        }
    }
    result.dedup();
    result
}

fn preferred_local_ipv4()->Result<Ipv4Addr,String>{
    local_ipv4_candidates().into_iter().next().ok_or_else(||"Nessun indirizzo IPv4 LAN utilizzabile trovato. Collegati a una rete Wi-Fi/Ethernet e riprova.".to_string())
}

fn same_lan(client:Ipv4Addr,local:Ipv4Addr)->bool{
    if client.is_loopback(){return true;}
    let a=client.octets();
    let b=local.octets();
    a[0]==b[0]&&a[1]==b[1]&&a[2]==b[2]
}

fn system_device_name()->String{
    env::var("COMPUTERNAME").or_else(|_|env::var("HOSTNAME")).ok().filter(|value|!value.trim().is_empty()).unwrap_or_else(||"_davSHARE Desktop".to_string())
}

fn default_receive_dir()->PathBuf{
    let home=env::var("USERPROFILE").or_else(|_|env::var("HOME")).unwrap_or_else(|_|".".to_string());
    PathBuf::from(home).join("Downloads").join("_davSHARE")
}

fn token()->String{
    rand::thread_rng().sample_iter(&Alphanumeric).take(18).map(char::from).collect()
}

fn header(name:&str,value:&str)->Header{
    Header::from_bytes(name.as_bytes(),value.as_bytes()).expect("valid HTTP header")
}

fn html_escape(value:&str)->String{
    value.replace('&',"&amp;").replace('<',"&lt;").replace('>',"&gt;").replace('"',"&quot;").replace('\'',"&#39;")
}

fn js_escape(value:&str)->String{
    value.replace('\\',"\\\\").replace('\'',"\\'").replace('\n',"\\n").replace('\r',"\\r")
}

fn ascii_header_filename(value:&str)->String{
    let cleaned:String=value.chars().map(|ch|if ch.is_ascii_alphanumeric()||matches!(ch,'.'|'_'|'-'|' '){ch}else{'_'}).collect();
    let cleaned=cleaned.trim().trim_matches('.').trim().to_string();
    if cleaned.is_empty(){"download".to_string()}else{cleaned}
}

fn rfc5987_filename(value:&str)->String{
    let mut result=String::new();
    for byte in value.as_bytes(){
        let ch=*byte as char;
        if ch.is_ascii_alphanumeric()||matches!(ch,'!'|'#'|'$'|'&'|'+'|'-'|'.'|'^'|'_'|'`'|'|'|'~'){
            result.push(ch);
        }else{
            result.push_str(&format!("%{:02X}",byte));
        }
    }
    result
}

fn percent_decode(value:&str)->String{
    let bytes=value.as_bytes();
    let mut output=Vec::with_capacity(bytes.len());
    let mut index=0;
    while index<bytes.len(){
        if bytes[index]==b'%'&&index+2<bytes.len(){
            let high=(bytes[index+1] as char).to_digit(16);
            let low=(bytes[index+2] as char).to_digit(16);
            if let (Some(high),Some(low))=(high,low){output.push(((high<<4)|low) as u8);index+=3;continue;}
        }
        output.push(if bytes[index]==b'+'{b' '}else{bytes[index]});
        index+=1;
    }
    String::from_utf8_lossy(&output).into_owned()
}

fn safe_filename(value:&str)->String{
    let decoded=percent_decode(value);
    let name=Path::new(&decoded).file_name().and_then(|part|part.to_str()).unwrap_or("file");
    let mut cleaned:String=name.chars().map(|ch|{
        if ch.is_control()||matches!(ch,'<'|'>'|':'|'"'|'/'|'\\'|'|'|'?'|'*'){ '_' }else{ch}
    }).collect();
    cleaned=cleaned.trim().trim_matches(|ch|ch=='.'||ch==' ').to_string();
    if cleaned.is_empty(){cleaned="file".to_string();}
    let stem=Path::new(&cleaned).file_stem().and_then(|part|part.to_str()).unwrap_or(&cleaned).to_ascii_uppercase();
    let reserved=matches!(stem.as_str(),"CON"|"PRN"|"AUX"|"NUL")||((stem.starts_with("COM")||stem.starts_with("LPT"))&&stem[3..].parse::<u8>().map(|n|(1..=9).contains(&n)).unwrap_or(false));
    if reserved{cleaned=format!("_{cleaned}");}
    if cleaned.chars().count()>180{
        let path=Path::new(&cleaned);
        let ext=path.extension().and_then(|part|part.to_str()).unwrap_or("");
        let stem=path.file_stem().and_then(|part|part.to_str()).unwrap_or("file");
        let max_stem=if ext.is_empty(){180}else{180usize.saturating_sub(ext.chars().count()+1)};
        let short:String=stem.chars().take(max_stem).collect();
        cleaned=if ext.is_empty(){short}else{format!("{short}.{ext}")};
    }
    cleaned
}

fn unique_destination(dir:&Path,name:&str)->PathBuf{
    let direct=dir.join(name);
    if !direct.exists(){return direct;}
    let path=Path::new(name);
    let stem=path.file_stem().and_then(|value|value.to_str()).unwrap_or("file");
    let ext=path.extension().and_then(|value|value.to_str()).unwrap_or("");
    for index in 2..10000{
        let candidate=if ext.is_empty(){dir.join(format!("{stem} ({index})"))}else{dir.join(format!("{stem} ({index}).{ext}"))};
        if !candidate.exists(){return candidate;}
    }
    dir.join(format!("{stem}-davshare"))
}

fn directory_size(path:&Path)->u64{
    let Ok(entries)=fs::read_dir(path)else{return 0;};
    entries.filter_map(Result::ok).map(|entry|{
        let p=entry.path();
        match fs::symlink_metadata(&p){
            Ok(meta) if meta.file_type().is_symlink()=>0,
            Ok(meta) if meta.is_file()=>meta.len(),
            Ok(meta) if meta.is_dir()=>directory_size(&p),
            _=>0,
        }
    }).sum()
}

fn inspect_paths(paths:Vec<String>)->Vec<ShareFile>{
    paths.into_iter().filter_map(|raw|{
        let path=PathBuf::from(&raw);
        let metadata=fs::symlink_metadata(&path).ok()?;
        if metadata.file_type().is_symlink(){return None;}
        let name=path.file_name().and_then(|value|value.to_str()).unwrap_or(&raw).to_string();
        if metadata.is_file(){Some(ShareFile{path:raw,name,size:metadata.len(),kind:"file".to_string()})}
        else if metadata.is_dir(){Some(ShareFile{path:raw,name,size:directory_size(&path),kind:"folder".to_string()})}
        else{None}
    }).collect()
}

fn zip_dir_recursive<W:Write+io::Seek>(zip:&mut ZipWriter<W>,root:&Path,current:&Path,options:FileOptions)->Result<(),String>{
    let entries=fs::read_dir(current).map_err(|error|error.to_string())?;
    for entry in entries{
        let entry=entry.map_err(|error|error.to_string())?;
        let path=entry.path();
        let meta=fs::symlink_metadata(&path).map_err(|error|error.to_string())?;
        if meta.file_type().is_symlink(){continue;}
        let relative=path.strip_prefix(root).map_err(|error|error.to_string())?.to_string_lossy().replace('\\',"/");
        if meta.is_dir(){
            zip.add_directory(format!("{relative}/"),options).map_err(|error|error.to_string())?;
            zip_dir_recursive(zip,root,&path,options)?;
        }else if meta.is_file(){
            zip.start_file(relative,options).map_err(|error|error.to_string())?;
            let mut input=File::open(&path).map_err(|error|error.to_string())?;
            io::copy(&mut input,zip).map_err(|error|error.to_string())?;
        }
    }
    Ok(())
}

fn prepare_share_files(paths:Vec<String>,session_token:&str)->Result<(Vec<ShareFile>,Vec<PathBuf>),String>{
    let mut files=Vec::new();
    let mut temp_paths=Vec::new();
    let temp_root=env::temp_dir().join("_davSHARE").join(session_token);
    for raw in paths{
        let path=PathBuf::from(&raw);
        let metadata=fs::symlink_metadata(&path).map_err(|error|error.to_string())?;
        if metadata.file_type().is_symlink(){continue;}
        if metadata.is_file(){
            files.push(ShareFile{path:raw.clone(),name:path.file_name().and_then(|value|value.to_str()).unwrap_or(&raw).to_string(),size:metadata.len(),kind:"file".to_string()});
        }else if metadata.is_dir(){
            fs::create_dir_all(&temp_root).map_err(|error|error.to_string())?;
            let base=path.file_name().and_then(|value|value.to_str()).unwrap_or("cartella");
            let archive=temp_root.join(format!("{}.zip",safe_filename(base)));
            let output=File::create(&archive).map_err(|error|error.to_string())?;
            let mut zip=ZipWriter::new(output);
            let options=FileOptions::default().compression_method(CompressionMethod::Deflated);
            zip_dir_recursive(&mut zip,&path,&path,options)?;
            zip.finish().map_err(|error|error.to_string())?;
            let size=fs::metadata(&archive).map_err(|error|error.to_string())?.len();
            files.push(ShareFile{path:archive.to_string_lossy().into_owned(),name:format!("{}.zip",base),size,kind:"archive".to_string()});
            temp_paths.push(archive);
        }
    }
    if temp_root.exists(){temp_paths.push(temp_root);}
    Ok((files,temp_paths))
}

fn cleanup_temp_paths(paths:&[PathBuf]){
    let mut retry=Vec::new();
    for path in paths.iter().rev(){
        let result=if path.is_dir(){fs::remove_dir_all(path)}else{fs::remove_file(path)};
        if result.is_err()&&path.exists(){retry.push(path.clone());}
    }
    if !retry.is_empty(){
        let _=thread::Builder::new().name("davshare-temp-cleanup".to_string()).spawn(move||{
            let mut pending=retry;
            for _ in 0..12{
                thread::sleep(Duration::from_millis(350));
                pending.retain(|path|{
                    let result=if path.is_dir(){fs::remove_dir_all(path)}else{fs::remove_file(path)};
                    result.is_err()&&path.exists()
                });
                if pending.is_empty(){break;}
            }
        });
    }
}

fn send_text(request:Request,status:u16,content_type:&str,body:String){
    let response=Response::from_string(body)
        .with_status_code(StatusCode(status))
        .with_header(header("Content-Type",content_type))
        .with_header(header("Cache-Control","no-store"))
        .with_header(header("X-Content-Type-Options","nosniff"))
        .with_header(header("Referrer-Policy","no-referrer"))
        .with_header(header("X-Frame-Options","DENY"))
        .with_header(header("Content-Security-Policy","default-src 'self'; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline'; connect-src 'self'; img-src 'self' data:; font-src https://fonts.gstatic.com"));
    let _=request.respond(response);
}

fn client_ip(request:&Request)->Option<Ipv4Addr>{
    match request.remote_addr().map(|address|address.ip()){
        Some(IpAddr::V4(ip))=>Some(ip),
        _=>None,
    }
}

fn session_snapshot(shared:&Arc<Mutex<SharedState>>,requested_token:&str)->Option<Session>{
    let mut guard=shared.lock().ok()?;
    let expired=guard.session.as_ref().map(|session|session.expires_at<=now()).unwrap_or(false);
    if expired{
        if let Some(old)=guard.session.take(){cleanup_temp_paths(&old.temp_paths);}
        return None;
    }
    guard.session.as_ref().filter(|session|session.token==requested_token).cloned()
}

fn session_is_active(shared:&Arc<Mutex<SharedState>>,session_token:&str)->bool{
    shared.lock().ok().and_then(|guard|guard.session.as_ref().map(|session|session.token==session_token&&session.expires_at>now())).unwrap_or(false)
}

fn client_authorized(session:&Session,ip:&str)->bool{
    !session.require_confirmation||session.approved_clients.contains(ip)
}

fn register_pending_client(shared:&Arc<Mutex<SharedState>>,session_token:&str,ip:&str){
    if let Ok(mut guard)=shared.lock(){
        if let Some(session)=guard.session.as_mut(){
            if session.token==session_token&&!session.approved_clients.contains(ip)&&!session.denied_clients.contains(ip){
                session.pending_clients.entry(ip.to_string()).or_insert_with(now);
                session.last_activity=now();
            }
        }
    }
}

fn add_history(guard:&mut SharedState,entry:HistoryEntry){
    guard.history.insert(0,entry);
    guard.history.truncate(HISTORY_LIMIT);
}

fn register_transfer(shared:&Arc<Mutex<SharedState>>,session_token:&str,name:&str,direction:&str,client:&str,total:u64)->Option<String>{
    let id=token();
    let transfer=TransferProgress{id:id.clone(),name:name.to_string(),direction:direction.to_string(),client_ip:client.to_string(),transferred:0,total,started_at:now()};
    let mut guard=shared.lock().ok()?;
    let session=guard.session.as_mut()?;
    if session.token!=session_token{return None;}
    session.transfers.insert(id.clone(),transfer);
    session.last_activity=now();
    Some(id)
}

fn update_transfer(shared:&Arc<Mutex<SharedState>>,session_token:&str,transfer_id:&str,bytes:u64,direction:&str)->bool{
    let mut guard=match shared.lock(){Ok(value)=>value,Err(_)=>return false};
    let Some(session)=guard.session.as_mut()else{return false;};
    if session.token!=session_token||session.expires_at<=now(){return false;}
    let Some(transfer)=session.transfers.get_mut(transfer_id)else{return false;};
    transfer.transferred=transfer.transferred.saturating_add(bytes);
    if direction=="send"{session.bytes_sent=session.bytes_sent.saturating_add(bytes);}else{session.bytes_received=session.bytes_received.saturating_add(bytes);}
    session.last_activity=now();
    true
}

fn finish_transfer(shared:&Arc<Mutex<SharedState>>,session_token:&str,transfer_id:&str,status:&str){
    let mut guard=match shared.lock(){Ok(value)=>value,Err(_)=>return};
    let mut history=None;
    if let Some(session)=guard.session.as_mut(){
        if session.token==session_token{
            if let Some(transfer)=session.transfers.remove(transfer_id){
                if status=="completed"{
                    if transfer.direction=="send"{session.downloads=session.downloads.saturating_add(1);}else{session.uploads=session.uploads.saturating_add(1);}
                }
                history=Some(HistoryEntry{id:transfer.id,name:transfer.name,direction:transfer.direction,client_ip:transfer.client_ip,size:transfer.transferred,completed_at:now(),status:status.to_string()});
                session.last_activity=now();
            }
        }
    }
    if let Some(entry)=history{add_history(&mut guard,entry);}
}

struct ProgressReader{
    file:File,
    shared:Arc<Mutex<SharedState>>,
    session_token:String,
    transfer_id:String,
    completed:bool,
}

impl Read for ProgressReader{
    fn read(&mut self,buf:&mut [u8])->io::Result<usize>{
        if !session_is_active(&self.shared,&self.session_token){return Err(io::Error::new(io::ErrorKind::Interrupted,"session closed"));}
        let read=self.file.read(buf)?;
        if read==0{
            if !self.completed{finish_transfer(&self.shared,&self.session_token,&self.transfer_id,"completed");self.completed=true;}
            return Ok(0);
        }
        if !update_transfer(&self.shared,&self.session_token,&self.transfer_id,read as u64,"send"){
            return Err(io::Error::new(io::ErrorKind::Interrupted,"session closed"));
        }
        Ok(read)
    }
}

impl Drop for ProgressReader{
    fn drop(&mut self){if !self.completed{finish_transfer(&self.shared,&self.session_token,&self.transfer_id,"interrupted");}}
}

fn mobile_session_watcher(session_token:&str)->String{
    let template=r#"<script>window.__davShareTransfers=window.__davShareTransfers||[];const davSessionToken='__TOKEN__';let davSessionWatching=true;let davSessionFailures=0;function davEndSession(){if(!davSessionWatching)return;davSessionWatching=false;for(const x of window.__davShareTransfers){try{x.abort()}catch{}}window.__davShareTransfers=[];const content=document.getElementById('session-content');const ended=document.getElementById('ended-screen');if(content)content.classList.add('session-hidden');if(ended)ended.classList.remove('session-hidden');document.title='_davSHARE';}async function davWatchSession(){if(!davSessionWatching)return;try{const r=await fetch('/s/'+davSessionToken+'/state?t='+Date.now(),{cache:'no-store'});if(r.ok){const j=await r.json();davSessionFailures=0;if(!j.active){davEndSession();return;}}else{davSessionFailures++;}}catch{davSessionFailures++;}if(davSessionFailures>=8){davEndSession();return;}setTimeout(davWatchSession,650)}setTimeout(davWatchSession,450);</script>"#;
    template.replace("__TOKEN__",&js_escape(session_token))
}

fn mobile_waiting_page(session:&Session,client:&str)->String{
    let english=session.language=="en";
    let title=if english{"Waiting for approval"}else{"In attesa di approvazione"};
    let body=if english{"Confirm this device from _davSHARE on the computer. This page will continue automatically."}else{"Conferma questo dispositivo da _davSHARE sul computer. Questa pagina continuerà automaticamente."};
    let ended_title=if english{"Session ended"}else{"Sessione terminata"};
    let ended_body=if english{"The sharing session was closed from the computer. Generate a new QR to reconnect."}else{"La sessione di condivisione è stata chiusa dal computer. Genera un nuovo QR per riconnetterti."};
    let watcher=mobile_session_watcher(&session.token);
    format!(r#"<!doctype html><html lang="{}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover"><meta name="color-scheme" content="light dark"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin><link rel="preload" href="https://fonts.gstatic.com/s/plusjakartasans/v11/LDIoaomQNQcsA88c7O9yZ4KMCoOg4Ko20yw.woff2" as="font" type="font/woff2" crossorigin><title>_davSHARE</title><style>@font-face{{font-family:"Plus Jakarta Sans";font-style:normal;font-weight:200 800;font-display:block;src:url("https://fonts.gstatic.com/s/plusjakartasans/v11/LDIoaomQNQcsA88c7O9yZ4KMCoOg4Ko20yw.woff2") format("woff2");unicode-range:U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD}}:root{{--accent:#006edb;--bg:#f7f7fa;--surface:#fff;--text:#1d1d1f;--muted:#71717a;--border:rgba(29,29,31,.12)}}@media(prefers-color-scheme:dark){{:root{{--accent:#2997ff;--bg:#0b0b0d;--surface:#151517;--text:#f5f5f7;--muted:#a1a1a6;--border:rgba(245,245,247,.12)}}}}*{{box-sizing:border-box}}body,button,input{{font-family:"Plus Jakarta Sans",system-ui,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;font-synthesis:none;text-rendering:optimizeLegibility;-webkit-font-smoothing:antialiased;-webkit-text-size-adjust:100%;text-size-adjust:100%;font-kerning:normal}}html.dav-font-loading body{{opacity:0}}html.dav-font-ready body{{opacity:1;transition:opacity .16s ease}}body{{margin:0;background:var(--bg);color:var(--text)}}main{{width:min(600px,100%);margin:auto;padding:32px 18px}}.brand{{font-weight:800;font-size:22px;letter-spacing:-.04em;margin-bottom:40px}}.brand span{{color:var(--accent)}}.card{{border:1px solid var(--border);background:var(--surface);border-radius:22px;padding:34px;text-align:center;box-shadow:0 18px 60px rgba(0,0,0,.05)}}.pulse,.ended-icon{{width:62px;height:62px;border-radius:18px;background:color-mix(in srgb,var(--accent) 12%,var(--surface));margin:0 auto 18px;display:grid;place-items:center;color:var(--accent);font-size:26px}}.pulse{{animation:p 1.6s ease-in-out infinite}}h1{{font-size:28px;letter-spacing:-.045em;margin:0 0 8px}}p{{color:var(--muted);line-height:1.55;margin:0}}small{{display:block;color:var(--muted);margin-top:18px}}.session-hidden{{display:none!important}}#ended-screen{{animation:ended .35s cubic-bezier(.2,.8,.2,1) both}}@keyframes p{{50%{{transform:scale(1.06);opacity:.72}}}}@keyframes ended{{from{{opacity:0;transform:translateY(8px) scale(.99)}}to{{opacity:1;transform:none}}}}</style><script>document.documentElement.classList.add('dav-font-loading');const davShowFont=()=>{{document.documentElement.classList.remove('dav-font-loading');document.documentElement.classList.add('dav-font-ready')}};const davFontUrl='https://fonts.gstatic.com/s/plusjakartasans/v11/LDIoaomQNQcsA88c7O9yZ4KMCoOg4Ko20yw.woff2';async function davLoadJakarta(){{try{{if('FontFace' in window&&document.fonts){{const face=new FontFace('Plus Jakarta Sans','url('+davFontUrl+') format("woff2")',{{style:'normal',weight:'200 800',display:'block'}});await face.load();document.fonts.add(face);await Promise.all([document.fonts.load('400 16px "Plus Jakarta Sans"'),document.fonts.load('800 16px "Plus Jakarta Sans"')]);if(document.fonts.check('400 16px "Plus Jakarta Sans"')){{document.documentElement.dataset.davFont='jakarta';davShowFont();return;}}}}}}catch{{}}document.documentElement.dataset.davFont='fallback';davShowFont();}}Promise.race([davLoadJakarta(),new Promise(r=>setTimeout(()=>{{document.documentElement.dataset.davFont='fallback';davShowFont();r();}},6500))]);</script></head><body><main id="session-content"><div class="brand"><span>_dav</span>SHARE</div><section class="card"><div class="pulse">⌁</div><h1>{}</h1><p>{}</p><small>{}</small></section></main><main id="ended-screen" class="session-hidden"><div class="brand"><span>_dav</span>SHARE</div><section class="card"><div class="ended-icon">×</div><h1>{}</h1><p>{}</p></section></main>{}<script>const token='{}';const client='{}';async function poll(){{if(!davSessionWatching)return;try{{const r=await fetch('/s/'+token+'/approval?client='+encodeURIComponent(client),{{cache:'no-store'}});if(!r.ok){{setTimeout(poll,900);return;}}const j=await r.json();if(j.approved){{location.reload();return;}}if(j.denied){{document.querySelector('#session-content h1').textContent={};document.querySelector('#session-content p').textContent={};return;}}}}catch{{}}setTimeout(poll,900)}}poll();</script></body></html>"#,
        if english{"en"}else{"it"},title,body,html_escape(client),ended_title,ended_body,watcher,js_escape(&session.token),js_escape(client),
        if english{"'Access denied'"}else{"'Accesso rifiutato'"},if english{"'The computer rejected this connection.'"}else{"'Il computer ha rifiutato questa connessione.'"})
}

fn mobile_page(session:&Session)->String{
    let en=session.language=="en";
    let mode_receive=session.mode=="receive";
    let title=if mode_receive{if en{"Send files to the computer"}else{"Invia file al computer"}}else{if en{"Download from the computer"}else{"Scarica dal computer"}};
    let subtitle=if mode_receive{if en{"Select photos, videos or documents. They stay on your local network."}else{"Seleziona foto, video o documenti. Rimangono nella rete locale."}}else{if en{"Tap a file to download it directly from the computer."}else{"Tocca un file per scaricarlo direttamente dal computer."}};
    let content=if mode_receive{
        let choose=if en{"Choose files"}else{"Scegli file"};
        let send_to=if en{"Send to"}else{"Invia a"};
        let hint=if en{"Choose one or more files from this device."}else{"Scegli uno o più file dal dispositivo."};
        let sending=if en{"Sending"}else{"Invio"};
        let error=if en{"Upload error"}else{"Errore durante l’invio"};
        let complete=if en{"files sent successfully."}else{"file inviati correttamente."};
        format!(r#"<section class="card upload-card"><div class="drop"><div class="icon">↑</div><h2>{} {}</h2><p>{}</p><input id="files" type="file" multiple><button id="pick">{}</button><div id="progress" class="progress hidden"><div id="bar"></div></div><p id="status" class="status"></p></div></section><script>const input=document.getElementById('files'),pick=document.getElementById('pick'),status=document.getElementById('status'),progress=document.getElementById('progress'),bar=document.getElementById('bar');pick.onclick=()=>input.click();input.onchange=async()=>{{const files=[...input.files];if(!files.length)return;progress.classList.remove('hidden');const total=files.reduce((s,f)=>s+f.size,0);let base=0,done=0;for(const file of files){{if(!davSessionWatching)return;status.textContent='{} '+file.name+'…';const ok=await new Promise(resolve=>{{const x=new XMLHttpRequest();window.__davShareTransfers=window.__davShareTransfers||[];window.__davShareTransfers.push(x);const cleanup=()=>{{window.__davShareTransfers=window.__davShareTransfers.filter(item=>item!==x)}};x.open('POST','/s/{}/upload?name='+encodeURIComponent(file.name));x.setRequestHeader('Content-Type','application/octet-stream');x.upload.onprogress=e=>{{if(e.lengthComputable)bar.style.width=Math.round((base+e.loaded)/total*100)+'%'}};x.onload=()=>{{cleanup();resolve(x.status>=200&&x.status<300)}};x.onerror=()=>{{cleanup();resolve(false)}};x.onabort=()=>{{cleanup();resolve(false)}};x.send(file);}});if(!ok){{if(davSessionWatching)status.textContent='{}: '+file.name;return;}}base+=file.size;done++;bar.style.width=Math.round(base/total*100)+'%';}}status.textContent=done+' {}';input.value='';}}</script>"#,send_to,html_escape(&session.device_name),hint,choose,sending,js_escape(&session.token),error,complete)
    }else{
        let download=if en{"Download"}else{"Scarica"};
        let available=if en{"Available files"}else{"File disponibili"};
        let empty=if en{"No files available."}else{"Nessun file disponibile."};
        let rows=if session.files.is_empty(){format!("<div class=\"empty\">{empty}</div>")}else{session.files.iter().enumerate().map(|(index,file)|format!(r#"<a class="file" href="/s/{}/download/{}"><div class="file-icon">↓</div><div><strong>{}</strong><span>{}</span></div><b>{}</b></a>"#,html_escape(&session.token),index,html_escape(&file.name),format_bytes(file.size),download)).collect::<Vec<_>>().join("")};
        format!(r#"<section class="card files"><div class="section-title"><h2>{}</h2><span>{} file</span></div>{}</section>"#,available,session.files.len(),rows)
    };
    let local=if en{"Local connection"}else{"Connessione locale"};
    let privacy=if en{"Temporary session. No file passes through _davstudios cloud or external servers."}else{"Sessione temporanea. Nessun file passa attraverso cloud o server esterni _davstudios."};
    let ended_title=if en{"Session ended"}else{"Sessione terminata"};
    let ended_body=if en{"The sharing session was closed from the computer. Generate a new QR to reconnect."}else{"La sessione di condivisione è stata chiusa dal computer. Genera un nuovo QR per riconnetterti."};
    let watcher=mobile_session_watcher(&session.token);
    format!(r#"<!doctype html><html lang="{}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover"><meta name="color-scheme" content="light dark"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin><link rel="preload" href="https://fonts.gstatic.com/s/plusjakartasans/v11/LDIoaomQNQcsA88c7O9yZ4KMCoOg4Ko20yw.woff2" as="font" type="font/woff2" crossorigin><title>_davSHARE</title><style>@font-face{{font-family:"Plus Jakarta Sans";font-style:normal;font-weight:200 800;font-display:block;src:url("https://fonts.gstatic.com/s/plusjakartasans/v11/LDIoaomQNQcsA88c7O9yZ4KMCoOg4Ko20yw.woff2") format("woff2");unicode-range:U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD}}:root{{--accent:#006edb;--bg:#f7f7fa;--surface:#fff;--text:#1d1d1f;--muted:#71717a;--border:rgba(29,29,31,.12)}}@media(prefers-color-scheme:dark){{:root{{--accent:#2997ff;--bg:#0b0b0d;--surface:#151517;--text:#f5f5f7;--muted:#a1a1a6;--border:rgba(245,245,247,.12)}}}}*{{box-sizing:border-box}}body,button,input{{font-family:"Plus Jakarta Sans",system-ui,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;font-synthesis:none;text-rendering:optimizeLegibility;-webkit-font-smoothing:antialiased;-webkit-text-size-adjust:100%;text-size-adjust:100%;font-kerning:normal}}html.dav-font-loading body{{opacity:0}}html.dav-font-ready body{{opacity:1;transition:opacity .16s ease}}body{{margin:0;background:var(--bg);color:var(--text)}}main{{width:min(680px,100%);margin:auto;padding:28px 18px 48px}}.brand{{font-weight:800;font-size:22px;letter-spacing:-.04em;margin-bottom:34px}}.brand span{{color:var(--accent)}}.hero{{margin-bottom:22px}}.hero small{{font-size:12px;color:var(--accent);font-weight:750;text-transform:uppercase;letter-spacing:.08em}}h1{{font-size:34px;line-height:1.08;letter-spacing:-.045em;margin:7px 0 9px}}.hero p{{color:var(--muted);line-height:1.55;margin:0}}.card{{background:var(--surface);border:1px solid var(--border);border-radius:22px;padding:18px;box-shadow:0 18px 60px rgba(0,0,0,.05)}}.section-title{{display:flex;justify-content:space-between;align-items:center;margin-bottom:12px}}.section-title h2{{font-size:18px;margin:0}}.section-title span{{font-size:12px;color:var(--muted)}}.file{{display:grid;grid-template-columns:44px minmax(0,1fr) auto;gap:12px;align-items:center;text-decoration:none;color:var(--text);padding:12px 0;border-top:1px solid var(--border)}}.file:first-of-type{{border-top:0}}.file-icon,.icon,.ended-icon{{display:grid;place-items:center;background:color-mix(in srgb,var(--accent) 11%,var(--surface));color:var(--accent);border-radius:12px}}.file-icon{{width:42px;height:42px;font-size:20px}}.file strong,.file span{{display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}}.file strong{{font-size:14px}}.file span{{font-size:11px;color:var(--muted);margin-top:3px}}.file b{{font-size:12px;color:var(--accent)}}.drop{{text-align:center;padding:20px 6px}}.icon{{width:64px;height:64px;margin:0 auto 16px;font-size:30px}}.drop h2{{font-size:22px;margin:0 0 6px}}.drop p{{font-size:13px;color:var(--muted);margin:0 0 18px}}input{{display:none}}button{{border:0;border-radius:12px;padding:13px 18px;background:var(--accent);color:#fff;font-weight:700;font-size:14px}}.progress{{height:6px;background:var(--border);border-radius:99px;overflow:hidden;margin-top:20px}}.progress div{{height:100%;width:0;background:var(--accent);transition:width .18s}}.hidden{{display:none}}.status{{min-height:20px;margin-top:12px!important}}.privacy{{display:flex;gap:9px;align-items:flex-start;color:var(--muted);font-size:11px;line-height:1.5;margin:18px 4px 0}}.dot{{width:8px;height:8px;border-radius:50%;background:#22a65a;margin-top:4px;flex:0 0 8px}}.empty{{color:var(--muted);text-align:center;padding:28px 8px}}.session-hidden{{display:none!important}}#ended-screen{{animation:ended .35s cubic-bezier(.2,.8,.2,1) both}}.ended-card{{padding:34px;text-align:center}}.ended-icon{{width:64px;height:64px;margin:0 auto 18px;font-size:29px}}.ended-card h1{{font-size:28px;margin-bottom:8px}}.ended-card p{{color:var(--muted);line-height:1.55;margin:0}}@keyframes ended{{from{{opacity:0;transform:translateY(8px) scale(.99)}}to{{opacity:1;transform:none}}}}</style><script>document.documentElement.classList.add('dav-font-loading');const davShowFont=()=>{{document.documentElement.classList.remove('dav-font-loading');document.documentElement.classList.add('dav-font-ready')}};const davFontUrl='https://fonts.gstatic.com/s/plusjakartasans/v11/LDIoaomQNQcsA88c7O9yZ4KMCoOg4Ko20yw.woff2';async function davLoadJakarta(){{try{{if('FontFace' in window&&document.fonts){{const face=new FontFace('Plus Jakarta Sans','url('+davFontUrl+') format("woff2")',{{style:'normal',weight:'200 800',display:'block'}});await face.load();document.fonts.add(face);await Promise.all([document.fonts.load('400 16px "Plus Jakarta Sans"'),document.fonts.load('800 16px "Plus Jakarta Sans"')]);if(document.fonts.check('400 16px "Plus Jakarta Sans"')){{document.documentElement.dataset.davFont='jakarta';davShowFont();return;}}}}}}catch{{}}document.documentElement.dataset.davFont='fallback';davShowFont();}}Promise.race([davLoadJakarta(),new Promise(r=>setTimeout(()=>{{document.documentElement.dataset.davFont='fallback';davShowFont();r();}},6500))]);</script></head><body><main id="session-content"><div class="brand"><span>_dav</span>SHARE</div><section class="hero"><small>{}</small><h1>{}</h1><p>{}</p></section>{}<div class="privacy"><span class="dot"></span><span>{}</span></div></main><main id="ended-screen" class="session-hidden"><div class="brand"><span>_dav</span>SHARE</div><section class="card ended-card"><div class="ended-icon">×</div><h1>{}</h1><p>{}</p></section></main>{}</body></html>"#,if en{"en"}else{"it"},local,title,subtitle,content,privacy,ended_title,ended_body,watcher)
}

fn expired_page(language:&str)->String{
    let en=language=="en";
    let title=if en{"Session ended"}else{"Sessione terminata"};
    let body=if en{"Generate a new QR from _davSHARE to reconnect."}else{"Genera un nuovo QR da _davSHARE per riconnetterti."};
    format!(r#"<!doctype html><html lang="{}"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover"><meta name="color-scheme" content="light dark"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin><link rel="preload" href="https://fonts.gstatic.com/s/plusjakartasans/v11/LDIoaomQNQcsA88c7O9yZ4KMCoOg4Ko20yw.woff2" as="font" type="font/woff2" crossorigin><title>_davSHARE</title><style>@font-face{{font-family:"Plus Jakarta Sans";font-style:normal;font-weight:200 800;font-display:block;src:url("https://fonts.gstatic.com/s/plusjakartasans/v11/LDIoaomQNQcsA88c7O9yZ4KMCoOg4Ko20yw.woff2") format("woff2");unicode-range:U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD}}:root{{--accent:#006edb;--bg:#f7f7fa;--surface:#fff;--text:#1d1d1f;--muted:#71717a;--border:rgba(29,29,31,.12)}}@media(prefers-color-scheme:dark){{:root{{--accent:#2997ff;--bg:#0b0b0d;--surface:#151517;--text:#f5f5f7;--muted:#a1a1a6;--border:rgba(245,245,247,.12)}}}}*{{box-sizing:border-box}}body{{margin:0;background:var(--bg);color:var(--text);font-family:"Plus Jakarta Sans",system-ui,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;font-synthesis:none;text-rendering:optimizeLegibility;-webkit-font-smoothing:antialiased;-webkit-text-size-adjust:100%;text-size-adjust:100%;font-kerning:normal}}html.dav-font-loading body{{opacity:0}}html.dav-font-ready body{{opacity:1;transition:opacity .16s ease}}main{{width:min(600px,100%);margin:auto;padding:32px 18px}}.brand{{font-weight:800;font-size:22px;letter-spacing:-.04em;margin-bottom:40px}}.brand span{{color:var(--accent)}}.card{{border:1px solid var(--border);background:var(--surface);border-radius:22px;padding:34px;text-align:center;box-shadow:0 18px 60px rgba(0,0,0,.05)}}.icon{{width:62px;height:62px;border-radius:18px;background:color-mix(in srgb,var(--accent) 12%,var(--surface));margin:0 auto 18px;display:grid;place-items:center;color:var(--accent);font-size:26px}}h1{{font-size:28px;letter-spacing:-.045em;margin:0 0 8px}}p{{color:var(--muted);line-height:1.55;margin:0}}</style><script>document.documentElement.classList.add('dav-font-loading');const davShowFont=()=>{{document.documentElement.classList.remove('dav-font-loading');document.documentElement.classList.add('dav-font-ready')}};const davFontUrl='https://fonts.gstatic.com/s/plusjakartasans/v11/LDIoaomQNQcsA88c7O9yZ4KMCoOg4Ko20yw.woff2';async function davLoadJakarta(){{try{{if('FontFace' in window&&document.fonts){{const face=new FontFace('Plus Jakarta Sans','url('+davFontUrl+') format("woff2")',{{style:'normal',weight:'200 800',display:'block'}});await face.load();document.fonts.add(face);await Promise.all([document.fonts.load('400 16px "Plus Jakarta Sans"'),document.fonts.load('800 16px "Plus Jakarta Sans"')]);if(document.fonts.check('400 16px "Plus Jakarta Sans"')){{document.documentElement.dataset.davFont='jakarta';davShowFont();return;}}}}}}catch{{}}document.documentElement.dataset.davFont='fallback';davShowFont();}}Promise.race([davLoadJakarta(),new Promise(r=>setTimeout(()=>{{document.documentElement.dataset.davFont='fallback';davShowFont();r();}},6500))]);</script></head><body><main><div class="brand"><span>_dav</span>SHARE</div><section class="card"><div class="icon">×</div><h1>{}</h1><p>{}</p></section></main></body></html>"#,if en{"en"}else{"it"},title,body)
}

fn format_bytes(size:u64)->String{
    let units=["B","KB","MB","GB","TB"];
    let mut value=size as f64;
    let mut index=0usize;
    while value>=1024.0&&index<units.len()-1{value/=1024.0;index+=1;}
    if index==0{format!("{} {}",size,units[index])}else{format!("{:.1} {}",value,units[index])}
}

fn available_space(path:&Path)->u64{
    fs2::available_space(path).unwrap_or(0)
}

fn handle_download(request:Request,shared:&Arc<Mutex<SharedState>>,session:&Session,index:usize,client:&str){
    if session.mode!="send"{send_text(request,403,"text/plain; charset=utf-8","Download non consentito in questa sessione".to_string());return;}
    if !client_authorized(session,client){send_text(request,403,"text/plain; charset=utf-8","Dispositivo non approvato".to_string());return;}
    let Some(item)=session.files.get(index).cloned()else{send_text(request,404,"text/plain; charset=utf-8","File non trovato".to_string());return;};
    let path=PathBuf::from(&item.path);
    let Ok(file)=File::open(&path)else{send_text(request,404,"text/plain; charset=utf-8","File non più disponibile".to_string());return;};
    let Some(transfer_id)=register_transfer(shared,&session.token,&item.name,"send",client,item.size)else{send_text(request,410,"text/plain; charset=utf-8","Sessione non disponibile".to_string());return;};
    let fallback=ascii_header_filename(&item.name);
    let encoded=rfc5987_filename(&item.name);
    let reader=ProgressReader{file,shared:Arc::clone(shared),session_token:session.token.clone(),transfer_id,completed:false};
    let response=Response::new(
        StatusCode(200),
        vec![header("Content-Type","application/octet-stream"),header("Content-Disposition",&format!("attachment; filename=\"{}\"; filename*=UTF-8''{}",fallback,encoded)),header("Cache-Control","no-store"),header("X-Content-Type-Options","nosniff"),header("Referrer-Policy","no-referrer")],
        reader,
        usize::try_from(item.size).ok(),
        None,
    );
    let _=request.respond(response);
}

fn handle_upload(mut request:Request,shared:&Arc<Mutex<SharedState>>,session:&Session,query:&str,client:&str){
    if session.mode!="receive"{send_text(request,403,"text/plain; charset=utf-8","Upload non consentito in questa sessione".to_string());return;}
    if !client_authorized(session,client){send_text(request,403,"text/plain; charset=utf-8","Dispositivo non approvato".to_string());return;}
    let expected=request.body_length().map(|value|value as u64).unwrap_or(0);
    let max=if session.max_upload_bytes==0{DEFAULT_MAX_UPLOAD_BYTES}else{session.max_upload_bytes};
    if expected>max{send_text(request,413,"text/plain; charset=utf-8","File oltre il limite configurato".to_string());return;}
    let free=available_space(&session.receive_dir);
    if expected>0&&free>0&&expected.saturating_add(MIN_FREE_RESERVE_BYTES)>free{send_text(request,507,"text/plain; charset=utf-8","Spazio libero insufficiente".to_string());return;}
    let name=query.split('&').find_map(|pair|pair.split_once('=').filter(|(key,_)|*key=="name").map(|(_,value)|safe_filename(value))).unwrap_or_else(||"file".to_string());
    if fs::create_dir_all(&session.receive_dir).is_err(){send_text(request,500,"text/plain; charset=utf-8","Impossibile creare la cartella di ricezione".to_string());return;}
    let target=unique_destination(&session.receive_dir,&name);
    let Ok(mut output)=File::create(&target)else{send_text(request,500,"text/plain; charset=utf-8","Impossibile creare il file".to_string());return;};
    let Some(transfer_id)=register_transfer(shared,&session.token,&name,"receive",client,expected)else{let _=fs::remove_file(&target);send_text(request,410,"text/plain; charset=utf-8","Sessione non disponibile".to_string());return;};
    let mut total=0u64;
    let mut buffer=[0u8;64*1024];
    loop{
        if !session_is_active(shared,&session.token){let _=fs::remove_file(&target);finish_transfer(shared,&session.token,&transfer_id,"interrupted");return;}
        let read=match request.as_reader().read(&mut buffer){Ok(value)=>value,Err(_)=>{let _=fs::remove_file(&target);finish_transfer(shared,&session.token,&transfer_id,"interrupted");send_text(request,500,"text/plain; charset=utf-8","Errore durante la ricezione".to_string());return;}};
        if read==0{break;}
        total=total.saturating_add(read as u64);
        if total>max{let _=fs::remove_file(&target);finish_transfer(shared,&session.token,&transfer_id,"rejected");send_text(request,413,"text/plain; charset=utf-8","File oltre il limite configurato".to_string());return;}
        if total%(8*1024*1024)<read as u64{
            let free=available_space(&session.receive_dir);
            if free>0&&free<MIN_FREE_RESERVE_BYTES{let _=fs::remove_file(&target);finish_transfer(shared,&session.token,&transfer_id,"rejected");send_text(request,507,"text/plain; charset=utf-8","Spazio libero insufficiente".to_string());return;}
        }
        if output.write_all(&buffer[..read]).is_err(){let _=fs::remove_file(&target);finish_transfer(shared,&session.token,&transfer_id,"interrupted");send_text(request,500,"text/plain; charset=utf-8","Errore durante il salvataggio".to_string());return;}
        if !update_transfer(shared,&session.token,&transfer_id,read as u64,"receive"){let _=fs::remove_file(&target);return;}
    }
    if expected>0&&expected!=total{let _=fs::remove_file(&target);finish_transfer(shared,&session.token,&transfer_id,"interrupted");send_text(request,400,"text/plain; charset=utf-8","Trasferimento incompleto".to_string());return;}
    if let Ok(mut guard)=shared.lock(){if let Some(current)=guard.session.as_mut(){if current.token==session.token{current.received.push(ReceivedFile{name:target.file_name().and_then(|value|value.to_str()).unwrap_or(&name).to_string(),path:target.to_string_lossy().into_owned(),size:total});}}}
    finish_transfer(shared,&session.token,&transfer_id,"completed");
    send_text(request,200,"application/json; charset=utf-8","{\"ok\":true}".to_string());
}

fn handle_request(request:Request,shared:&Arc<Mutex<SharedState>>){
    let Some(remote_ip)=client_ip(&request)else{send_text(request,403,"text/plain; charset=utf-8","Client non valido".to_string());return;};
    let raw=request.url().to_string();
    let (path,query)=raw.split_once('?').unwrap_or((&raw,""));
    let values=path.split('/').filter(|value|!value.is_empty()).collect::<Vec<_>>();
    if values.len()<2||values[0]!="s"{send_text(request,404,"text/plain; charset=utf-8","_davSHARE: sessione non trovata".to_string());return;}
    if values.len()==3&&values[2]=="state"&&request.method()==&Method::Get{
        let active=session_snapshot(shared,values[1]).is_some();
        send_text(request,200,"application/json; charset=utf-8",format!("{{\"active\":{}}}",active));return;
    }
    let Some(session)=session_snapshot(shared,values[1])else{send_text(request,410,"text/html; charset=utf-8",expired_page("it"));return;};
    if !same_lan(remote_ip,session.local_ip){send_text(request,403,"text/plain; charset=utf-8","Accesso consentito solo dalla stessa rete locale".to_string());return;}
    let remote=remote_ip.to_string();
    if values.len()==3&&values[2]=="approval"&&request.method()==&Method::Get{
        let (approved,denied)=match shared.lock(){Ok(guard)=>guard.session.as_ref().filter(|current|current.token==session.token).map(|current|(current.approved_clients.contains(&remote),current.denied_clients.contains(&remote))).unwrap_or((false,false)),Err(_)=>(false,false)};
        send_text(request,200,"application/json; charset=utf-8",format!("{{\"approved\":{},\"denied\":{}}}",approved,denied));return;
    }
    if values.len()==2&&request.method()==&Method::Get{
        if session.require_confirmation&&!session.approved_clients.contains(&remote){
            if session.denied_clients.contains(&remote){send_text(request,403,"text/plain; charset=utf-8","Accesso rifiutato".to_string());return;}
            register_pending_client(shared,&session.token,&remote);
            send_text(request,200,"text/html; charset=utf-8",mobile_waiting_page(&session,&remote));return;
        }
        send_text(request,200,"text/html; charset=utf-8",mobile_page(&session));return;
    }
    if values.len()==4&&values[2]=="download"&&request.method()==&Method::Get{
        if let Ok(index)=values[3].parse::<usize>(){handle_download(request,shared,&session,index,&remote);}else{send_text(request,400,"text/plain; charset=utf-8","Indice non valido".to_string());}
        return;
    }
    if values.len()==3&&values[2]=="upload"&&request.method()==&Method::Post{handle_upload(request,shared,&session,query,&remote);return;}
    send_text(request,404,"text/plain; charset=utf-8","Risorsa non trovata".to_string());
}

impl Runtime{
    fn ensure_server(&self)->Result<(Ipv4Addr,u16),String>{
        if let Some((stored_ip,port))=*self.server_binding.lock().map_err(|_|"Stato server non disponibile".to_string())?{return Ok((preferred_local_ipv4().unwrap_or(stored_ip),port));}
        let ip=preferred_local_ipv4()?;
        for port in DEFAULT_PORT..DEFAULT_PORT+40{
            if let Ok(server)=Server::http(format!("0.0.0.0:{port}")){
                let shared=Arc::clone(&self.shared);
                thread::Builder::new().name("davshare-http".to_string()).spawn(move||{
                    for request in server.incoming_requests(){
                        let shared=Arc::clone(&shared);
                        let _=thread::Builder::new().name("davshare-transfer".to_string()).spawn(move||handle_request(request,&shared));
                    }
                }).map_err(|error|error.to_string())?;
                *self.server_binding.lock().map_err(|_|"Stato server non disponibile".to_string())?=Some((ip,port));
                return Ok((ip,port));
            }
        }
        Err("Nessuna porta locale disponibile per _davSHARE".to_string())
    }
}

#[tauri::command]
fn runtime_info(state:tauri::State<'_,Runtime>)->Result<RuntimeInfo,String>{
    let (ip,port)=state.ensure_server()?;
    Ok(RuntimeInfo{local_ip:ip.to_string(),port,interface_ips:local_ipv4_candidates().into_iter().map(|value|value.to_string()).collect(),device_name:system_device_name(),default_receive_dir:default_receive_dir().to_string_lossy().into_owned()})
}

#[tauri::command]
fn network_diagnostics(state:tauri::State<'_,Runtime>)->Result<NetworkDiagnostics,String>{
    let (ip,port)=state.ensure_server()?;
    let candidates=local_ipv4_candidates();
    let self_test=TcpStream::connect_timeout(&format!("{ip}:{port}").parse().map_err(|_|"Indirizzo locale non valido".to_string())?,Duration::from_millis(500)).is_ok();
    Ok(NetworkDiagnostics{bound_ip:ip.to_string(),port,interface_ips:candidates.iter().map(|value|value.to_string()).collect(),self_test,network_changed:!candidates.contains(&ip)})
}

#[tauri::command]
async fn inspect_share_files(paths:Vec<String>)->Result<Vec<ShareFile>,String>{
    tauri::async_runtime::spawn_blocking(move||inspect_paths(paths)).await.map_err(|error|error.to_string())
}

#[tauri::command]
async fn start_share_session(state:tauri::State<'_,Runtime>,mode:String,paths:Vec<String>,receive_dir:String,device_name:String,language:String,require_confirmation:bool,max_upload_bytes:u64,network_ip:String)->Result<SessionView,String>{
    if mode!="send"&&mode!="receive"{return Err("Modalità sessione non valida".to_string());}
    let (auto_ip,port)=state.ensure_server()?;
    let candidates=local_ipv4_candidates();
    let requested=network_ip.trim();
    let ip=if requested.is_empty()||requested=="auto"{auto_ip}else{
        let parsed=requested.parse::<Ipv4Addr>().map_err(|_|"Indirizzo LAN selezionato non valido".to_string())?;
        if !candidates.contains(&parsed){return Err("L'indirizzo LAN selezionato non è più disponibile. Scegli Automatico o un altro IP nelle Impostazioni.".to_string());}
        parsed
    };
    let session_token=token();
    let prepared_token=session_token.clone();
    let (files,temp_paths)=if mode=="send"{
        tauri::async_runtime::spawn_blocking(move||prepare_share_files(paths,&prepared_token)).await.map_err(|error|error.to_string())??
    }else{(Vec::new(),Vec::new())};
    if mode=="send"&&files.is_empty(){return Err("Seleziona almeno un file o una cartella da condividere".to_string());}
    let receive=if receive_dir.trim().is_empty(){default_receive_dir()}else{PathBuf::from(receive_dir.trim())};
    if mode=="receive"{fs::create_dir_all(&receive).map_err(|error|error.to_string())?;}
    let expires_at=now()+SESSION_TTL_SECONDS;
    let name=if device_name.trim().is_empty(){system_device_name()}else{device_name.trim().to_string()};
    let lang=if language=="en"{"en".to_string()}else{"it".to_string()};
    let url=format!("http://{}:{}/s/{}",ip,port,session_token);
    let code=QrCode::new(url.as_bytes()).map_err(|error|error.to_string())?;
    let qr_svg=code.render::<svg::Color>().min_dimensions(320,320).dark_color(svg::Color("#111114")).light_color(svg::Color("#ffffff")).build();
    let session=Session{token:session_token.clone(),mode:mode.clone(),device_name:name,language:lang,files:files.clone(),receive_dir:receive.clone(),expires_at,downloads:0,uploads:0,bytes_sent:0,bytes_received:0,received:Vec::new(),last_activity:now(),local_ip:ip,require_confirmation,max_upload_bytes:if max_upload_bytes==0{DEFAULT_MAX_UPLOAD_BYTES}else{max_upload_bytes},pending_clients:HashMap::new(),approved_clients:HashSet::new(),denied_clients:HashSet::new(),transfers:HashMap::new(),temp_paths};
    let mut guard=state.shared.lock().map_err(|_|"Stato sessione non disponibile".to_string())?;
    if let Some(old)=guard.session.take(){cleanup_temp_paths(&old.temp_paths);}
    guard.session=Some(session);
    Ok(SessionView{active:true,mode,url,qr_svg,token:session_token,expires_at,files,receive_dir:receive.to_string_lossy().into_owned()})
}

fn inactive_status(history:Vec<HistoryEntry>)->ShareStatus{
    ShareStatus{active:false,mode:String::new(),expires_at:0,downloads:0,uploads:0,bytes_sent:0,bytes_received:0,received:Vec::new(),last_activity:0,active_transfers:Vec::new(),pending_clients:Vec::new(),history}
}

#[tauri::command]
fn share_status(state:tauri::State<'_,Runtime>)->ShareStatus{
    let mut guard=match state.shared.lock(){Ok(value)=>value,Err(_)=>return inactive_status(Vec::new())};
    if guard.session.as_ref().map(|session|session.expires_at<=now()).unwrap_or(false){if let Some(old)=guard.session.take(){cleanup_temp_paths(&old.temp_paths);}}
    let history=guard.history.clone();
    if let Some(session)=guard.session.as_ref(){
        let mut transfers=session.transfers.values().cloned().collect::<Vec<_>>();
        transfers.sort_by_key(|item|item.started_at);
        let mut pending=session.pending_clients.iter().map(|(ip,requested_at)|PendingClient{ip:ip.clone(),requested_at:*requested_at}).collect::<Vec<_>>();
        pending.sort_by_key(|item|item.requested_at);
        return ShareStatus{active:true,mode:session.mode.clone(),expires_at:session.expires_at,downloads:session.downloads,uploads:session.uploads,bytes_sent:session.bytes_sent,bytes_received:session.bytes_received,received:session.received.clone(),last_activity:session.last_activity,active_transfers:transfers,pending_clients:pending,history};
    }
    inactive_status(history)
}

#[tauri::command]
fn approve_client(state:tauri::State<'_,Runtime>,ip:String)->Result<(),String>{
    let mut guard=state.shared.lock().map_err(|_|"Stato sessione non disponibile".to_string())?;
    let session=guard.session.as_mut().ok_or_else(||"Nessuna sessione attiva".to_string())?;
    session.pending_clients.remove(&ip);
    session.denied_clients.remove(&ip);
    session.approved_clients.insert(ip);
    session.last_activity=now();
    Ok(())
}

#[tauri::command]
fn reject_client(state:tauri::State<'_,Runtime>,ip:String)->Result<(),String>{
    let mut guard=state.shared.lock().map_err(|_|"Stato sessione non disponibile".to_string())?;
    let session=guard.session.as_mut().ok_or_else(||"Nessuna sessione attiva".to_string())?;
    session.pending_clients.remove(&ip);
    session.approved_clients.remove(&ip);
    session.denied_clients.insert(ip);
    session.last_activity=now();
    Ok(())
}

#[tauri::command]
fn stop_share_session(state:tauri::State<'_,Runtime>)->Result<(),String>{
    let mut guard=state.shared.lock().map_err(|_|"Stato sessione non disponibile".to_string())?;
    if let Some(old)=guard.session.take(){cleanup_temp_paths(&old.temp_paths);}
    Ok(())
}

#[tauri::command]
fn reveal_path(path:String)->Result<(),String>{
    let target=PathBuf::from(path);
    let folder=if target.is_dir(){target}else{target.parent().unwrap_or(Path::new(".")).to_path_buf()};
    #[cfg(target_os="windows")]
    let mut command={let mut cmd=Command::new("explorer");cmd.arg(folder);cmd};
    #[cfg(target_os="macos")]
    let mut command={let mut cmd=Command::new("open");cmd.arg(folder);cmd};
    #[cfg(all(unix,not(target_os="macos")))]
    let mut command={let mut cmd=Command::new("xdg-open");cmd.arg(folder);cmd};
    command.spawn().map_err(|error|error.to_string())?;
    Ok(())
}

#[cfg_attr(mobile,tauri::mobile_entry_point)]
pub fn run(){
    tauri::Builder::default()
        .manage(Runtime::default())
        .setup(|app|{
            #[cfg(target_os="windows")]
            {
                if let Some(window)=app.get_webview_window("main"){window.set_icon(tauri::include_image!("./icons/icon.ico"))?;}
            }
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![runtime_info,network_diagnostics,inspect_share_files,start_share_session,share_status,approve_client,reject_client,stop_share_session,reveal_path])
        .run(tauri::generate_context!())
        .expect("error while running _davSHARE");
}
