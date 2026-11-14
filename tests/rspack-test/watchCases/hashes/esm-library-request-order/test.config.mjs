import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

let output;
let initialShared;

export default {
  findBundle(_, config) {
    output = config.output.path;
    return [];
  },
  checkStats(step, stats) {
    for (const file of ['setup.mjs', 'value.mjs']) {
      fs.copyFileSync(
        path.join(import.meta.dirname, '0', file),
        path.join(output, file),
      );
    }

    const assets = stats.entrypoints.main.assets.map(({ name }) => name);
    const main = assets.find((name) => name.startsWith('main.'));
    const shared = assets.find((name) => name.startsWith('shared.'));
    const url = pathToFileURL(path.join(output, main)).href;
    const stdout = execFileSync(
      process.execPath,
      [
        '--input-type=module',
        '-e',
        `const { value } = await import(${JSON.stringify(url)}); console.log(value);`,
      ],
      { encoding: 'utf8' },
    );
    expect(stdout.trim()).toBe(step === '1' ? 'true' : 'false');
    if (step === '0') initialShared = shared;
    else if (step === '1') expect(shared).not.toBe(initialShared);
    else expect(shared).toBe(initialShared);
    return true;
  },
};
