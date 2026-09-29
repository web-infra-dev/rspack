import { defineConfig } from '@rspack/cli';
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const dir = path.resolve(import.meta.dirname, 'temp');
const file = path.resolve(dir, 'index.js');

fs.mkdirSync(dir, {
  recursive: true,
});
fs.writeFileSync(
  file,
  `import v1 from ${JSON.stringify(
    pathToFileURL(
      path.resolve(import.meta.dirname, './src with spaces/module.js'),
    ),
  )};
import v2 from ${JSON.stringify(
    'file://localhost' +
      pathToFileURL(
        path.resolve(import.meta.dirname, './src with spaces/module.js'),
      )
        .toString()
        .slice('file://'.length),
  )};
export const val1 = v1;
export const val2 = v2;
`,
);
fs.utimesSync(file, new Date(Date.now() - 10000), new Date(Date.now() - 10000));

export default defineConfig({
  mode: 'development',
  target: 'node',
});
