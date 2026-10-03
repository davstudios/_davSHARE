use crate::core::{result, ActionOptions, ActionResult};
use local_ip_address::local_ip;
use std::{fs, io::{Read, Write}, net::{TcpListener, TcpStream}, path::PathBuf, thread};
use uuid::Uuid;

fn respond(mut stream:TcpStream,status:&str,content_type:&str,body:Vec<u8>){let head=format!("HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",status,content_type,body.len());let _=stream.write_all(head.as_bytes());let _=stream.write_all(&body);}
fn serve(mut stream:TcpStream,folder:PathBuf,token:String){let mut buf=[0u8;8192];let n=stream.read(&mut buf).unwrap_or(0);let req=String::from_utf8_lossy(&buf[..n]);let first=req.lines().next().unwrap_or("");let parts=first.split_whitespace().collect::<Vec<_>>();if parts.len()<2{return;}let target=parts[1];if !target.contains(&format!("token={}",token)){respond(stream,"403 Forbidden","text/plain",b"Invalid token".to_vec());return;}if target.starts_with("/file/"){let name=target.trim_start_matches("/file/").split('?').next().unwrap_or("");if name.contains("..")||name.contains('/')||name.contains('\\'){respond(stream,"400 Bad Request","text/plain",b"Invalid path".to_vec());return;}let path=folder.join(name);match fs::read(path){Ok(data)=>respond(stream,"200 OK","application/octet-stream",data),Err(_)=>respond(stream,"404 Not Found","text/plain",b"Not found".to_vec())}return;}let mut links=String::new();if let Ok(entries)=fs::read_dir(&folder){for e in entries.filter_map(Result::ok){if e.path().is_file(){let n=e.file_name().to_string_lossy().into_owned();links.push_str(&format!("<li><a href=\"/file/{}?token={}\">{}</a></li>",n,token,n));}}}let html=format!("<!doctype html><meta charset=utf-8><title>_davSHARE</title><style>body{{font:16px system-ui;max-width:720px;margin:50px auto;padding:20px}}a{{color:#006edb}}</style><h1>_davSHARE</h1><p>Local session</p><ul>{}</ul>",links);respond(stream,"200 OK","text/html; charset=utf-8",html.into_bytes());}

#[tauri::command]
pub fn run_action(action:String,paths:Vec<String>,_options:ActionOptions)->ActionResult{
    if action!="start_share"{return result(false,"Preview feature","This sharing workflow will be enabled after LAN testing",action);}
    let Some(folder)=paths.first()else{return result(false,"Folder required","Choose the folder to share",String::new());};
    let listener=match TcpListener::bind("0.0.0.0:0"){Ok(v)=>v,Err(e)=>return result(false,"Server failed","Unable to bind a LAN port",e.to_string())};let port=listener.local_addr().map(|a|a.port()).unwrap_or(0);let ip=local_ip().map(|v|v.to_string()).unwrap_or_else(|_|"127.0.0.1".into());let token=Uuid::new_v4().simple().to_string();let folder=PathBuf::from(folder);let thread_token=token.clone();thread::spawn(move||{for stream in listener.incoming().filter_map(Result::ok){serve(stream,folder.clone(),thread_token.clone());}});let url=format!("http://{}:{}/?token={}",ip,port,token);result(true,"LAN session started","Open this address from a device on the same network",url)
}

