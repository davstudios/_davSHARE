import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import crypto from 'node:crypto';

function pngColorType(path){const data=fs.readFileSync(path);assert.equal(data.subarray(1,4).toString(),'PNG');return data[25];}
for(const name of ['32x32.png','128x128.png','128x128@2x.png','app-icon.png'])test(`${name} usa PNG RGBA`,()=>assert.equal(pngColorType(`src-tauri/icons/${name}`),6));

test('icon.ico corrisponde all’asset definitivo _davSHARE',()=>{const data=fs.readFileSync('src-tauri/icons/icon.ico');assert.equal(crypto.createHash('sha256').update(data).digest('hex'),'1f946caebbba93eacde9d846a73a3b04c76ef508046a6f8d506022893b5c60f0');});
test('icon.icns macOS è presente e valido',()=>{const data=fs.readFileSync('src-tauri/icons/icon.icns');assert.equal(data.subarray(0,4).toString(),'icns');assert.ok(data.length>1024);});

