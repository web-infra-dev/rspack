import { defineConfig } from '@rspack/cli';
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
if (process.platform !== 'win32') {
  const fixtureDir = path.join(import.meta.dirname, 'foo\\bar');
  fs.mkdirSync(fixtureDir, { recursive: true });
  fs.writeFileSync(
    path.join(fixtureDir, 'a.js'),
    'module.exports = "backslash";\n',
  );
}

fs.writeFileSync(
  path.join(import.meta.dirname, 'posix-backslash.generated.js'),
  `import { createRequire as _createRequire } from "module";

it("should create require from absolute file URL object", () => {
\texpect(_createRequire(new URL(${JSON.stringify(pathToFileURL(path.join(import.meta.dirname, 'foo/c.js')).href)}))("./a")).toBe(4);
});

it("should create require from absolute file URL object with ignored base", () => {
\texpect(_createRequire(new URL(${JSON.stringify(pathToFileURL(path.join(import.meta.dirname, 'foo/c.js')).href)}, undefined))("./a")).toBe(4);
});

it("should normalize direct file URL dot segments", () => {
\texpect(_createRequire(${JSON.stringify(`${pathToFileURL(`${import.meta.dirname}${path.sep}`).href}foo/..`)})("./a")).toBe(1);
});

it("should accept normalized file URL object spellings", () => {
\texpect(_createRequire(new URL(${JSON.stringify(pathToFileURL(path.join(import.meta.dirname, 'foo/c.js')).href.replace('file:///', 'file:/'))}, import.meta.url))("./a")).toBe(4);
});
` +
    (process.platform === 'win32'
      ? '\n'
      : `

it("should treat POSIX absolute paths ending in backslash as files", () => {
\texpect(_createRequire(__dirname + "/foo\\\\")("./posix-backslash")).toBe(
\t\t"posix-backslash"
\t);
});
`),
);

export default defineConfig({
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  target: 'node',
  module: {
    parser: {
      javascript: {
        createRequire: true,
      },
    },
  },
  optimization: {
    inlineExports: true,
    moduleIds: 'named',
  },
});
