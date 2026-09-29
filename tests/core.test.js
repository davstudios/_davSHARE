import test from 'node:test';
import assert from 'node:assert/strict';
import { formatBytes, safeName } from '../src/helpers.js';
test('formats bytes',()=>{assert.equal(formatBytes(0),'0 B');assert.equal(formatBytes(1024),'1 KB');});
test('safeName keeps basename',()=>{assert.equal(safeName('C:/a/b/file.txt'),'file.txt');assert.equal(safeName('/a/b/file.txt'),'file.txt');});
