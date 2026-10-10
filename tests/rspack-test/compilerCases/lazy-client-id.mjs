import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import { lazyCompilationMiddleware } from '@rspack/core';

const nativeRequire = createRequire(import.meta.url);
export default ['back\\slash', 'double"quote'].map(escaped => {
  let directory;
  return {
  description: `escapes ${escaped} in the lazy client module ID`,
  options() {
    directory = fs.mkdtempSync(path.join(os.tmpdir(), 'rspack-lazy-client-'));
    fs.writeFileSync(path.join(directory, 'index.js'), "export const load = () => import('./lazy.js');");
    fs.writeFileSync(path.join(directory, 'lazy.js'), 'export const value = 42;');
    fs.writeFileSync(path.join(directory, 'client.js'), `exports.activate = ({ onError }) => {
      onError(new Error('escaped client reached'));
      return () => {};
    };`);
    return {
      mode: 'development',
      context: directory,
      target: 'node',
      entry: './index.js',
      devtool: false,
      output: { path: path.join(directory, 'dist'), filename: 'index.cjs', library: { type: 'commonjs2' } },
      lazyCompilation: {
        entries: false,
        imports: true,
        client: `${path.join(directory, 'client.js')}?escaped=${escaped}`,
        serverUrl: 'http://localhost:12345',
      },
    };
  },
  async build(context, compiler) {
    compiler.outputFileSystem = fs;
    lazyCompilationMiddleware(compiler);
    try {
      await new Promise((resolve, reject) => compiler.run((error, stats) => {
        if (error) return reject(error);
        if (stats.hasErrors()) return reject(new Error(stats.toString()));
        resolve();
      }));
      const runtime = nativeRequire(path.join(directory, 'dist', 'index.cjs'));
      await assert.rejects(runtime.load(), /escaped client reached/);
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
};
});
