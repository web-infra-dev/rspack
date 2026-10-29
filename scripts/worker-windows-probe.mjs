import { spawn } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
const helper = readFileSync('packages/rspack-test-tools/src/helper/node-case.ts', 'utf8');
const template = helper.split('const bootstrap = `')[1].split('`;')[0];
const resultPrefix = '__RSPACK_NODE_CASE_RESULT__:';
const bootstrap = eval('`' + template + '`');
let failures = 0;
async function probe(scenario, mode, preload, iteration) {
  const name = `${scenario}-${mode}-${preload}-${iteration}`;
  const moduleURL = new URL(`../tests/rspack-test/compilerCases/${scenario}/workers.mjs`, import.meta.url);
  const code = (preload ? `await import(${JSON.stringify(new URL('../crates/node_binding/binding.js', import.meta.url).href)});\n` : '') + bootstrap;
  await new Promise(resolve => {
    const child = spawn(process.execPath, ['--expose-gc', '--input-type=module', '--eval', code, moduleURL.href, JSON.stringify([mode])], { stdio: ['ignore', 'pipe', 'pipe'] });
    let stdout = '', stderr = '';
    const timer = setTimeout(() => child.kill(), 60000);
    child.stdout.on('data', data => stdout += data);
    child.stderr.on('data', data => stderr += data);
    child.on('close', (exitCode, signal) => {
      clearTimeout(timer);
      const result = { name, pid: child.pid, exitCode, signal, stdout, stderr };
      console.log(JSON.stringify(result));
      writeFileSync(`diagnostics/${name}.json`, JSON.stringify(result, null, 2));
      if (exitCode !== 0) failures++;
      resolve();
    });
  });
}
for (const preload of [false, true]) {
  failures = 0;
  for (let iteration = 0; iteration < 50; iteration++) {
    await Promise.all([probe('worker-errors', 'close', preload, iteration), probe('worker-references', 'close', preload, iteration), probe('worker-references', 'terminate', preload, iteration)]);
    if (failures >= 5) break;
  }
}
console.log(JSON.stringify({ failures }));
