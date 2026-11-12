import fs from 'node:fs';
import path from 'node:path';

const sourceDir = import.meta.dirname;

export default {
  beforeExecute(options) {
    const config = Array.isArray(options) ? options[0] : options;
    for (const name of ['ext.mjs', 'tail.mjs']) {
      fs.copyFileSync(
        path.join(sourceDir, name),
        path.join(config.output.path, name),
      );
    }
  },
  findBundle() {
    return ['main.mjs'];
  },
  async afterExecute(options) {
    const config = Array.isArray(options) ? options[0] : options;
    const mainUrl = path.join(config.output.path, 'main.mjs');
    await import(mainUrl);

    const source = fs.readFileSync(mainUrl, 'utf-8');
    const ordered = source
      .split('\n')
      .map((line) => line.trim())
      .filter(
        (line) => line.startsWith('import ') || line.startsWith('export '),
      );
    const setupIdx = ordered.findIndex((line) => line.includes('setup.mjs'));
    const extExportIdx = ordered.findIndex((line) =>
      line.includes('from "./ext.mjs"'),
    );
    expect(setupIdx).toBeGreaterThanOrEqual(0);
    expect(extExportIdx).toBeGreaterThanOrEqual(0);
    expect(setupIdx).toBeLessThan(extExportIdx);
  },
};
