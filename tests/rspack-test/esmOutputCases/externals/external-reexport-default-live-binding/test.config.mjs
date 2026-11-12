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
  async afterExecute(options) {
    const config = Array.isArray(options) ? options[0] : options;
    const mainUrl = path.join(config.output.path, 'main.mjs');
    const main = await import(mainUrl);
    expect(main.default).toBe(1);
    main.update();
    expect(main.default).toBe(2);

    const source = fs.readFileSync(mainUrl, 'utf-8');
    expect(source).not.toMatch(/export default value;/);
    expect(source).toMatch(/export \{[^}]*value as default/);
  },
};
