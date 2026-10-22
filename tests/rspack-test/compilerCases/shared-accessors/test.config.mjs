import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { runNodeCase } from '@rspack/test-tools/helper/node-case';

export default [
  {
    description:
      'resolves shared NormalModule errors against the receiver and current build',
    async run() {
      await runNodeCase(new URL('./errors.mjs', import.meta.url));
    },
  },
  {
    description: 'keeps shared accessors valid across rebuild, close and GC',
    async run() {
      await runNodeCase(new URL('./lifecycle.mjs', import.meta.url));
    },
  },
  {
    description:
      'isolates shared accessors and releases them when workers exit',
    async run() {
      await runNodeCase(new URL('./workers.mjs', import.meta.url));
    },
  },
  ...['natural', 'exit', 'worker'].flatMap((mode) =>
    [0, 1].map((code) => ({
      name: `shutdown-${mode}-${code}`,
      description: `does not report shared reference leaks on ${mode} shutdown with code ${code}`,
      run() {
        const result = spawnSync(
          process.execPath,
          [
            fileURLToPath(new URL('./exit.mjs', import.meta.url)),
            mode,
            String(code),
          ],
          { encoding: 'utf8', timeout: 20_000 },
        );
        assert.ifError(result.error);
        assert.equal(result.signal, null, result.stderr);
        assert.equal(result.status, code, result.stderr);
        assert.equal(result.stdout.trim(), 'compiler closed');
        assert.doesNotMatch(result.stderr, /ObjectRef is not unref/);
      },
    })),
  ),
];
