const fs = require('fs');
const path = require('path');
const { execFileSync } = require('child_process');

module.exports = {
  snapshotContent(content) {
    return content.replace(/[ \t]+$/gm, '').replace(/ +\t/g, '\t');
  },
  afterExecute(options) {
    const esmSource = fs.readFileSync(
      path.join(options.output.path, 'worker.mjs'),
      'utf-8',
    );

    expect(esmSource).not.toContain('registerModules');
    expect(esmSource).not.toMatch(
      /(?:__webpack_require__|__rspack_context\.r|rspackRequire)\("\.\/worker\.js"\);/,
    );
    expect(esmSource).toMatch(/^#!\/usr\/bin\/env node\n"use client"\n/);
    expect(esmSource).toContain('globalThis.__workerEntryExecuted = true;');

    const commonJsSource = fs.readFileSync(
      path.join(options.output.path, 'worker-cjs.mjs'),
      'utf-8',
    );

    expect(commonJsSource).toMatch(
      /(?:__webpack_require__|__rspack_context\.r|rspackRequire)\("\.\/worker-cjs\.js"\);/,
    );
    expect(commonJsSource).toContain(
      'globalThis.__workerCommonJsEntryExecuted = true;',
    );

    const asyncSource = fs.readFileSync(
      path.join(options.output.path, 'worker-async.mjs'),
      'utf-8',
    );

    expect(asyncSource).toMatch(
      /await (?:__webpack_require__\(__webpack_require__\.s|__rspack_context\.r\(__rspack_context\.s|rspackRequire\(entryModuleId) = "\.\/worker-async\.js"\);/,
    );
    expect(asyncSource).toContain(
      'globalThis.__workerAsyncEntryExecuted = true;',
    );

    execFileSync(process.execPath, [
      path.join(options.output.path, 'worker-async.mjs'),
    ]);
  },
};
