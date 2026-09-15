const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');

module.exports = {
  findBundle() {
    return [];
  },
  afterExecute(configurations) {
    for (const options of configurations) {
      const directory = path.join(options.output.path, options.name);
      const files = fs.readdirSync(directory).filter((file) => file.endsWith('.mjs'));
      const source = fs.readFileSync(path.join(directory, 'main.mjs'), 'utf-8');
      const enabled = options.output.library.type === 'modern-module';
      const shared = options.name.startsWith('shared-');

      expect(files).toHaveLength(shared ? 3 : enabled ? 2 : 1);
      if (enabled) {
        expect(source).toMatch(/export \* as directNs from /);
        expect(source).toMatch(/export \* as importedNs from /);
      } else {
        expect(source).not.toMatch(/export \* as /);
      }
    }
    const result = spawnSync(process.execPath, [
      path.join(__dirname, 'consumer-build.js'),
      path.join(configurations[0].output.path, 'modern-module'),
    ], { encoding: 'utf-8' });
    if (result.status !== 0) {
      throw new Error(result.error?.message || result.stderr || result.stdout);
    }
  },
};
