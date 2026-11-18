import fs from 'node:fs';
import path from 'node:path';
import { defineConfig } from '@rspack/cli';

const directory = fs.mkdtempSync(path.join(import.meta.dirname, 'tmp-'));
const fixture = path.join(directory, 'generated.js');
fs.writeFileSync(fixture, 'module.exports = "temporary fixture";');

export default defineConfig({
  cache: false,
  resolve: { alias: { 'temporary-fixture': fixture } },
});
