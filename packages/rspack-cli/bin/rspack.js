#!/usr/bin/env node
import nodeModule from 'node:module';

// enable on-disk code caching of all modules loaded by Node.js
// requires Nodejs >= 22.8.0
const { enableCompileCache } = nodeModule;
const isCI = Boolean(process.env.CI) && process.env.CI !== 'false';
// Skip CI, where builds typically run once and cannot reuse the cache.
if (enableCompileCache && !isCI) {
  try {
    enableCompileCache();
  } catch {
    // ignore errors
  }
}

async function runCLI() {
  const { RspackCLI } = await import('../dist/index.js');
  const cli = new RspackCLI();
  await cli.run(process.argv);
}

runCLI().catch((err) => {
  console.error('[rspack] CLI failed:');
  console.error(err);
  process.exit(1);
});
