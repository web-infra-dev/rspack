import { spawn } from 'node:child_process';
import { readFileSync, writeFileSync, readdirSync, mkdirSync } from 'node:fs';
const helper = readFileSync('packages/rspack-test-tools/src/helper/node-case.ts', 'utf8');
const template = helper.split('const bootstrap = `')[1].split('`;')[0];
const resultPrefix = '__RSPACK_NODE_CASE_RESULT__:';
const bootstrap = eval('`' + template + '`');
writeFileSync('diagnostics/bootstrap.mjs', bootstrap.replace('process.argv[1]', 'process.argv[2]').replace('JSON.parse(process.argv[2])', '[process.argv[3]]'));
let failures = 0;
async function probe(scenario, mode, preload, iteration) {
  const name = `${scenario}-${mode}-${preload}-${iteration}`;
  const moduleURL = new URL(`../tests/rspack-test/compilerCases/${scenario}/workers.mjs`, import.meta.url);
  const code = (preload ? `await import(${JSON.stringify(new URL('../crates/node_binding/binding.js', import.meta.url).href)});\n` : '') + bootstrap;
  mkdirSync(`diagnostics/${name}`, { recursive: true });
  await new Promise(resolve => {
    const child = spawn(process.env.PROCDUMP, ['-accepteula', '-mm', '-e', '1', '-f', 'C0000005', '-n', '1', '-x', `diagnostics/${name}`, process.execPath, '--expose-gc', 'diagnostics/bootstrap.mjs', moduleURL.href, mode], { stdio: ['ignore', 'pipe', 'pipe'] });
    let stdout = '', stderr = '';
    const timer = setTimeout(() => child.kill(), 60000);
    child.stdout.on('data', data => stdout += data);
    child.stderr.on('data', data => stderr += data);
    child.on('close', (exitCode, signal) => {
      clearTimeout(timer);
      const result = { name, pid: child.pid, exitCode, signal, stdout, stderr };
      console.log(JSON.stringify(result));
      writeFileSync(`diagnostics/${name}.json`, JSON.stringify(result, null, 2));
      if (!stdout.replaceAll('\0', '').includes('Exit Code 0x00000000')) failures++;
      resolve();
    });
  });
}
for (const preload of [false]) {
  failures = 0;
  for (let iteration = 0; iteration < 50; iteration++) {
    await Promise.all([probe('worker-errors', 'close', preload, iteration), probe('worker-references', 'close', preload, iteration), probe('worker-references', 'terminate', preload, iteration)]);
    if (readdirSync('diagnostics', { recursive: true }).filter(name => name.endsWith('.dmp')).length >= 2) break;
  }
}
console.log(JSON.stringify({ failures }));
