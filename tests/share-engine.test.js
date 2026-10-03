import test from 'node:test';
import assert from 'node:assert/strict';
import { formatBytes,totalBytes,uniqueFiles,remainingSeconds,formatRemaining } from '../src/share-engine.js';

test('formatBytes formatta dimensioni leggibili',()=>{assert.equal(formatBytes(0),'0 B');assert.equal(formatBytes(1024),'1.00 KB');assert.equal(formatBytes(1048576),'1.00 MB');});
test('totalBytes somma i file',()=>{assert.equal(totalBytes([{size:10},{size:20}]),30);});
test('uniqueFiles elimina duplicati per percorso',()=>{assert.equal(uniqueFiles([{path:'a'},{path:'a'},{path:'b'}]).length,2);});
test('remainingSeconds non scende sotto zero',()=>{assert.equal(remainingSeconds(100,90),10);assert.equal(remainingSeconds(90,100),0);});
test('formatRemaining usa minuti e secondi',()=>{assert.equal(formatRemaining(65),'1:05');});

test('transferRate calcola byte al secondo',async()=>{const {transferRate}=await import('../src/share-engine.js');assert.equal(transferRate(1000,10,12),500);});
test('transferEta stima il tempo rimanente',async()=>{const {transferEta}=await import('../src/share-engine.js');assert.equal(transferEta(1000,500,250),2);assert.equal(transferEta(1000,1000,250),0);});

