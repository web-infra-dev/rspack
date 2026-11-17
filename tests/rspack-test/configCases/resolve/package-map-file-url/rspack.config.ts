import { defineConfig } from '@rspack/cli';
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

export default defineConfig((_env, { testPath }) => {
  const url = (...segments: string[]) =>
    pathToFileURL(path.join(import.meta.dirname, ...segments)).href;
  // The package map is outside the project, so every package needs an absolute URL.
  const packageMap = path.resolve(testPath, 'package-map.json');
  fs.mkdirSync(testPath, { recursive: true });
  fs.writeFileSync(
    packageMap,
    JSON.stringify({
      packages: {
        app: {
          url: url('.'),
          dependencies: {
            dep: 'dep',
            'dep-with-space': 'dep-with-space',
            'dep-without-authority': 'dep-without-authority',
          },
        },
        dep: {
          // file:///path
          url: `${url('store', 'dep')}?query#fragment`,
        },
        'dep-with-space': {
          // file://localhost/path with a percent-encoded space
          url: url('store', 'dep with space').replace(
            'file:///',
            'file://localhost/',
          ),
        },
        'dep-without-authority': {
          // file:/path
          url: url('store', 'dep-without-authority').replace(
            'file:///',
            'file:/',
          ),
        },
      },
    }),
  );

  return {
    resolve: {
      packageMap,
    },
  };
});
