import fs from 'node:fs';
import path from 'node:path';

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
    const source = fs.readFileSync(
      path.join(config.output.path, 'main.mjs'),
      'utf-8',
    );
    const ordered = source
      .split('\n')
      .map((line) => line.trim())
      .filter(
        (line) => line.startsWith('import ') || line.startsWith('export '),
      );
    const sideImportIdx = ordered.findIndex((line) =>
      line.includes('./side.mjs'),
    );
    const extImportIdx = ordered.findIndex((line) =>
      line.includes('./ext.mjs'),
    );
    expect(sideImportIdx).toBeGreaterThanOrEqual(0);
    expect(extImportIdx).toBeGreaterThanOrEqual(0);
    expect(extImportIdx).toBeLessThan(sideImportIdx);
    expect(ordered.some((line) => line.includes('你好'))).toBe(true);
    expect(
      ordered.some(
        (line) =>
          line.includes('reservedDefault') && line.includes('extDefault'),
      ),
    ).toBe(true);
    expect(ordered.some((line) => line.includes('nested'))).toBe(true);
  },
};
