import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { defineConfig } from '@rspack/cli';
import { DefinePlugin } from '@rspack/core';

export default defineConfig(
  ['tsconfig.json', 'tsconfig-no-base-url.json'].flatMap((filename) => {
    const configFile = path.join(import.meta.dirname, filename);
    return [configFile, path.relative(process.cwd(), configFile)].flatMap(
      (configFile) =>
        [configFile, { configFile }].map((tsConfig) => ({
          resolve: { tsConfig },
          plugins: [
            new DefinePlugin({
              EXPECTED_URL: JSON.stringify(
                pathToFileURL(
                  path.join(import.meta.dirname, 'generated/client.js'),
                ).href,
              ),
            }),
          ],
        })),
    );
  }),
);
