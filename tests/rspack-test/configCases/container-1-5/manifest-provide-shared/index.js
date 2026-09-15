const fs = require('fs');
const path = require('path');

for (const suffix of ['', '-stats']) {
  it(`should retain the expose chunk when a local dependency is provided (${suffix || 'manifest'})`, () => {
    const data = JSON.parse(
      fs.readFileSync(path.join(__dirname, `mf-${CASE_INDEX}${suffix}.json`), 'utf-8'),
    );
    const expose = data.exposes.find(item => item.name === 'Mount');
    const exposeFile = `${CASE_INDEX}-expose-mount.js`;
    expect(expose.assets.js.sync).toEqual([exposeFile]);
    expect(fs.existsSync(path.join(__dirname, exposeFile))).toBe(true);
    expect(expose.assets.js.async).not.toContain(exposeFile);
    expect(expose.assets.css.sync).toEqual([`${CASE_INDEX}-expose-mount.css`]);

    if (PROVIDED_REQUEST === './static.js') {
      const shared = data.shared.find(item => item.name === PROVIDED_REQUEST);
      // The same file can contain both a local import and a provided module.
      expect(shared.assets.js.sync).toContain(exposeFile);
      expect(shared.assets.css.sync).toContain(`${CASE_INDEX}-expose-mount.css`);
      const providerFiles = shared.assets.js.sync.filter(file => file !== exposeFile);
      expect(providerFiles.length).toBeGreaterThan(0);
      for (const file of providerFiles) {
        expect(expose.assets.js.sync).not.toContain(file);
      }
    }
  });
}
