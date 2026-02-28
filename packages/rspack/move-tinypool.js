// This script copies the tinypool package from node_modules to the compiled directory.
// It removes the engines field from package.json and ensures that the package is working under `engine-strict=true` on lower node versions.
import fs from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';

const require = createRequire(import.meta.url);
const tinypool = path.dirname(require.resolve('tinypool/package.json'));
const dest = path.resolve(import.meta.dirname, './compiled/tinypool');

fs.cpSync(tinypool, dest, { recursive: true, force: true });

const pkg = JSON.parse(fs.readFileSync(path.join(dest, 'package.json')));
// Removes restrictions on node version (>= 18)
delete pkg.engines;
fs.writeFileSync(path.join(dest, 'package.json'), JSON.stringify(pkg, null, 2));
