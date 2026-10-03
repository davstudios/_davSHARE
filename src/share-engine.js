export function formatBytes(bytes){
  const value=Number(bytes)||0;
  if(value<1024)return `${value} B`;
  const units=['KB','MB','GB','TB'];
  let size=value/1024;
  let index=0;
  while(size>=1024&&index<units.length-1){size/=1024;index+=1;}
  return `${size>=100?size.toFixed(0):size>=10?size.toFixed(1):size.toFixed(2)} ${units[index]}`;
}

export function totalBytes(files){return (files||[]).reduce((sum,file)=>sum+(Number(file.size)||0),0);}

export function uniqueFiles(files){
  const seen=new Set();
  return (files||[]).filter((file)=>{const key=String(file.path||'');if(!key||seen.has(key))return false;seen.add(key);return true;});
}

export function remainingSeconds(expiresAt,nowSeconds=Math.floor(Date.now()/1000)){return Math.max(0,(Number(expiresAt)||0)-nowSeconds);}

export function formatRemaining(seconds){
  const value=Math.max(0,Math.floor(Number(seconds)||0));
  const minutes=Math.floor(value/60);
  const secs=value%60;
  return `${minutes}:${String(secs).padStart(2,'0')}`;
}

export function transferRate(bytes,startedAt,nowSeconds=Math.floor(Date.now()/1000)){
  const elapsed=Math.max(1,(Number(nowSeconds)||0)-(Number(startedAt)||0));
  return Math.max(0,(Number(bytes)||0)/elapsed);
}

export function transferEta(total,transferred,bytesPerSecond){
  const remaining=Math.max(0,(Number(total)||0)-(Number(transferred)||0));
  const rate=Number(bytesPerSecond)||0;
  if(!remaining||rate<=0)return 0;
  return Math.ceil(remaining/rate);
}

