import { defineConfig } from '@rspack/cli';
import path from 'node:path';
import fs from 'node:fs';
import { rspack } from '@rspack/core';

export default defineConfig((_env, { srcPath }) => {
  const valueFile = path.resolve(srcPath, 'value.txt');
  return {
    plugins: [
      new rspack.DefinePlugin({
        // @ts-expect-error Rspack does not support DefinePlugin.runtimeValue yet.
        TEST_VALUE: rspack.DefinePlugin.runtimeValue(() => {
          return JSON.stringify(fs.readFileSync(valueFile, 'utf-8').trim());
        }, [valueFile]),
        // @ts-expect-error Rspack does not support DefinePlugin.runtimeValue yet.
        TEST_VALUE2: rspack.DefinePlugin.runtimeValue(() => {
          return JSON.stringify(fs.readFileSync(valueFile, 'utf-8').trim());
        }, []),
        // @ts-expect-error Rspack does not support DefinePlugin.runtimeValue yet.
        TEST_VALUE3: rspack.DefinePlugin.runtimeValue(() => {
          return JSON.stringify(fs.readFileSync(valueFile, 'utf-8').trim());
        }, true),
        // @ts-expect-error Rspack does not support DefinePlugin.runtimeValue yet.
        TEST_VALUE4: rspack.DefinePlugin.runtimeValue(
          () => {
            return JSON.stringify(fs.readFileSync(valueFile, 'utf-8').trim());
          },
          {
            fileDependencies: [valueFile],
          },
        ),
        // @ts-expect-error Rspack does not support DefinePlugin.runtimeValue yet.
        TEST_VALUE5: rspack.DefinePlugin.runtimeValue(
          ({ version, key }: { version: string; key: string }) => {
            return JSON.stringify({ version, key });
          },
          {
            version: () => fs.readFileSync(valueFile, 'utf-8').trim(),
          },
        ),
      }),
    ],
  };
});
