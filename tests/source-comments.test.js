import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { dirname, extname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const exts=new Set(['.js','.css','.rs','.html']);
const excluded=new Set([resolve(root,'tests/source-comments.test.js')]);
function walk(path){if(!statSync(path).isDirectory())return[path];return readdirSync(path).flatMap((entry)=>walk(join(path,entry)));}
function stripQuoted(source){let result='';let quote='';let escaped=false;for(let i=0;i<source.length;i+=1){const c=source[i];if(quote){result+=c==='\n'?'\n':' ';if(escaped){escaped=false;continue;}if(c==='\\'){escaped=true;continue;}if(c===quote)quote='';continue;}if(c==='"'||c==="'"||c==='`'){quote=c;result+=' ';continue;}result+=c;}return result;}
function hasComment(path){const source=readFileSync(path,'utf8');if(extname(path)==='.html'&&source.includes('<!--'))return true;const code=stripQuoted(source);return code.includes('/*')||/(^|[^:])\/\//m.test(code);}
test('source files contain no code comments',()=>{const paths=[...walk(resolve(root,'src')),...walk(resolve(root,'src-tauri/src')),resolve(root,'src-tauri/build.rs'),resolve(root,'index.html'),resolve(root,'vite.config.js'),resolve(root,'tests/core.test.js')].filter((path)=>exts.has(extname(path))&&!excluded.has(path));assert.deepEqual(paths.filter(hasComment).map((p)=>p.slice(root.length+1)),[]);});

