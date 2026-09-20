import fs from 'node:fs/promises';
import path from 'node:path';
import { ContextModule } from '@rspack/core';

const initialTime = new Date('2000-01-01T00:00:00Z');
const modifiedTime = new Date('2000-01-02T00:00:00Z');
let compilerIndex = 0;
let contextIdentifier;

/** @type {import('@rspack/core').Configuration} */
export default {
  experiments: {
    newCache: {
      codeGeneration: false,
      devtool: false,
      loader: false,
      minimize: false,
      module: true,
    },
  },
  cache: {
    type: 'persistent',
    snapshot: {
      module: { timestamp: false, hash: true },
      contextModule: { timestamp: true, hash: false },
    },
  },
  plugins: [
    {
      apply(compiler) {
        const built = [];
        const restored = [];
        compiler.hooks.beforeRun.tapPromise(
          'ContextModuleTimestampTest',
          async () => {
            // Keep timestamps before build start so safe-time checks allow hits.
            const time = compilerIndex < 2 ? initialTime : modifiedTime;
            await fs.utimes(
              path.join(compiler.context, 'context/value.js'),
              time,
              time,
            );
          },
        );
        compiler.hooks.compilation.tap(
          'ContextModuleTimestampTest',
          (compilation) => {
            compilation.hooks.buildModule.tap(
              'ContextModuleTimestampTest',
              (module) => {
                if (module instanceof ContextModule)
                  built.push(module.identifier());
              },
            );
            compilation.hooks.stillValidModule.tap(
              'ContextModuleTimestampTest',
              (module) => {
                if (module instanceof ContextModule)
                  restored.push(module.identifier());
              },
            );
          },
        );
        compiler.hooks.done.tap('ContextModuleTimestampTest', () => {
          if (compilerIndex === 0) {
            expect(built).toHaveLength(1);
            contextIdentifier = built[0];
          }
          if (compilerIndex === 0 || compilerIndex === 2) {
            expect(built).toEqual([contextIdentifier]);
            expect(restored).toEqual([]);
          } else {
            expect(built).toEqual([]);
            expect(restored).toEqual([contextIdentifier]);
          }
          compilerIndex++;
        });
      },
    },
  ],
};
