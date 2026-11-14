import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const sourceDir = import.meta.dirname;

export default {
  beforeExecute(options) {
    const config = Array.isArray(options) ? options[0] : options;
    for (const name of ['ext.mjs', 'side.mjs']) {
      fs.copyFileSync(
        path.join(sourceDir, name),
        path.join(config.output.path, name),
      );
    }
  },
  findBundle() {
    return ['main.mjs'];
  },
  afterExecute(options) {
    const config = Array.isArray(options) ? options[0] : options;
    const mainUrl = path.join(config.output.path, 'main.mjs');
    const hashDir = path.join(config.output.path, 'out#frag');
    fs.mkdirSync(hashDir, { recursive: true });
    for (const name of ['main.mjs', 'ext.mjs', 'side.mjs']) {
      fs.copyFileSync(
        path.join(config.output.path, name),
        path.join(hashDir, name),
      );
    }
    const url = pathToFileURL(path.join(hashDir, 'main.mjs')).href;
    const result = execFileSync(
      process.execPath,
      [
        '--input-type=module',
        '-e',
        `const main = await import(${JSON.stringify(url)});
         const before = main.default;
         main.update();
         console.log(JSON.stringify([before, main.default]));`,
      ],
      { encoding: 'utf8' },
    );
    expect(JSON.parse(result)).toEqual([1, 2]);

    const source = fs.readFileSync(mainUrl, 'utf-8');
    expect(source).not.toMatch(/export default value;/);
    expect(source).toMatch(/export \{[^}]*value as default/);
  },
};
