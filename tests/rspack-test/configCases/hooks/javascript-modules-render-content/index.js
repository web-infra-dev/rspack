const fs = require('fs');
const path = require('path');

it('should let renderContent replace the runtime chunk content', () => {
  // The wrapper the hook returned is part of the emitted chunk...
  const content = fs.readFileSync(__filename, 'utf-8');
  expect(content).toContain('/* rendered:main */');
  expect(content).toContain('/* end */');
  // ...and it runs, so the hook output is what the chunk is built from rather
  // than something appended afterwards.
  expect(globalThis.__rendered_main__).toBe(true);
});

it('should let renderContent replace a non-runtime chunk content', async () => {
  const { value } = await import(/* webpackChunkName: "async" */ './async');
  expect(value).toBe('async-value');

  const chunk = fs.readFileSync(
    path.join(__dirname, 'async.chunk.js'),
    'utf-8',
  );
  expect(chunk).toContain('/* rendered:async */');
  expect(chunk).toContain('/* end */');
  expect(globalThis.__rendered_async__).toBe(true);
});
