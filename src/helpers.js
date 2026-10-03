export function formatBytes(value){const n=Number(value)||0;if(n<1024)return `${n} B`;const units=['KB','MB','GB','TB'];let size=n/1024;let unit=0;while(size>=1024&&unit<units.length-1){size/=1024;unit+=1;}return `${Number(size.toFixed(size>=10?1:2))} ${units[unit]}`;}
export function safeName(path){return String(path||'').split(/[\\/]/).filter(Boolean).at(-1)||'';}

