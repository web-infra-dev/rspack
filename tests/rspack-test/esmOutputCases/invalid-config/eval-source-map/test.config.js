const fs = require('node:fs');
const path = require('node:path');

module.exports = {
  snapshotFileFilter(file) {
    return file === 'main.mjs';
  },
  afterExecute(options) {
    const source = fs.readFileSync(
      path.join(options.output.path, 'main.mjs'),
      'utf-8',
    );

    expect(source).not.toContain('eval(');
    expect(source).toContain('sourceMappingURL=main.mjs.map');
    expect(fs.existsSync(path.join(options.output.path, 'main.mjs.map'))).toBe(
      true,
    );
  },
};
