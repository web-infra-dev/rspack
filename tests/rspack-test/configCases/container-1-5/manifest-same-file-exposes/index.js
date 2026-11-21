const fs = require('fs');
const path = require('path');

for (const filename of ['mf-stats.json', 'mf-manifest.json']) {
  it(`should retain both same-file aliases in ${filename}`, () => {
    const source = JSON.parse(
      fs.readFileSync(path.join(__dirname, filename), 'utf-8'),
    );
    expect(source.exposes).toHaveLength(2);
    expect(source.exposes.map((expose) => expose.path).sort()).toEqual([
      './client',
      './server',
    ]);
    for (const alias of ['server', 'client']) {
      const expose = source.exposes.find((item) => item.name === alias);
      expect(expose.id).toBe(`same_file_exposes:${alias}`);
      expect(expose.assets.js.sync).toContain(`${alias}.js`);
      expect(expose.assets.js.sync).not.toContain(
        `${alias === 'server' ? 'client' : 'server'}.js`,
      );
      if (filename === 'mf-stats.json') {
        expect(expose.file).toBe('module.js');
      }
    }
  });
}

it('should load both same-file aliases in their own layers', async () => {
  const container = require('./container-file.js');
  await container.init({});
  const server = (await container.get('./server'))();
  const client = (await container.get('./client'))();
  expect(server.layer).toBe('server');
  expect(client.layer).toBe('client');
  expect(server).not.toBe(client);
});

it('should validate layers on forwarded container options at runtime', () => {
  const { ContainerPlugin } = require('@rspack/core').container;
  const exposes = { './entry': { import: './module.js', layer: 'server' } };
  for (const enhanced of [undefined, false]) {
    expect(
      () => new ContainerPlugin({ name: 'forwarded', exposes, enhanced }),
    ).toThrow('layer requires enhanced=true');
  }
  expect(
    () => new ContainerPlugin({ name: 'forwarded', exposes, enhanced: true }),
  ).not.toThrow();
});
