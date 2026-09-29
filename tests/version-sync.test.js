import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';

const pkg=JSON.parse(fs.readFileSync('package.json','utf8'));
const tauri=JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json','utf8'));
const cargo=fs.readFileSync('src-tauri/Cargo.toml','utf8');
const main=fs.readFileSync('src/main.js','utf8');

test('versioni stabili sincronizzate',()=>{assert.equal(pkg.version,'1.0.1');assert.equal(tauri.version,'1.0.1');assert.equal(cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1],'1.0.1');assert.match(main,/version:'1\.0\.1'/);});
test('identità _davSHARE coerente',()=>{assert.equal(tauri.productName,'_davSHARE');assert.equal(tauri.identifier,'studio.dav.share');assert.equal(pkg.name,'davshare-app');});
test('interfaccia stabile non mostra Preview',()=>{assert.doesNotMatch(main,/· Preview/);});
