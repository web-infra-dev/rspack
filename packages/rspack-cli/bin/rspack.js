#!/usr/bin/env node
import nodeModule from 'node:module';

// enable on-disk code caching of all modules loaded by Node.js
// requires Nodejs >= 22.8.0
const { enableCompileCache } = nodeModule;
if (enableCompileCache) {
  try {
    enableCompileCache();
  } catch {
    // ignore errors
  }
}

// make it easier to identify the process via activity monitor or other tools
process.title = 'rspack-node';

import { RspackCLI } from '../dist/index.js';

async function runCLI() {
  const cli = new RspackCLI();
  await cli.run(process.argv);
}

runCLI();
