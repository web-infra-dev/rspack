const { execFileSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const { pathToFileURL } = require('url');

it('keeps setup mocks when a later test chunk installs its module factories', () => {
  const runtimeSource = fs.readFileSync(
    path.resolve(__dirname, 'rstest-runtime.mjs'),
    'utf-8',
  );

  expect(runtimeSource).toContain(
    '__webpack_require__.rstest_original_modules || {}',
  );
  expect(runtimeSource).toContain('for (moduleId in moreModules) {');

  execFileSync(
    process.execPath,
    [
      '--input-type=module',
      '--eval',
      `await import(${JSON.stringify(pathToFileURL(path.resolve(__dirname, 'rstest-runtime.mjs')).href)});\n` +
        `await import(${JSON.stringify(pathToFileURL(path.resolve(__dirname, 'setup.mjs')).href)});\n` +
        `await import(${JSON.stringify(pathToFileURL(path.resolve(__dirname, 'test.mjs')).href)});`,
    ],
    { stdio: 'pipe' },
  );
});
