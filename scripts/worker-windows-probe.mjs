import { writeSync } from 'node:fs';
import { setTimeout } from 'node:timers/promises';
const [scenario, mode, preload] = process.argv.slice(2);
writeSync(2, `PROBE start ${scenario} ${mode} preload=${preload}\n`);
if (preload === 'yes') {
  await import('../crates/node_binding/binding.js');
  writeSync(2, 'PROBE parent binding loaded\n');
}
const { default: run } = await import(`../tests/rspack-test/compilerCases/${scenario}/workers.mjs`);
await run(mode);
writeSync(2, 'PROBE scenario completed\n');
await setTimeout(100);
writeSync(2, 'PROBE grace completed\n');
