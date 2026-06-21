const fs = require('fs');
const path = require('path');
const { pathToFileURL } = require('url');

if (process.platform !== 'win32') {
  const fixtureDir = path.join(__dirname, 'foo\\bar');
  fs.mkdirSync(fixtureDir, { recursive: true });
  fs.writeFileSync(
    path.join(fixtureDir, 'a.js'),
    'module.exports = "backslash";\n',
  );
}

fs.writeFileSync(
  path.join(__dirname, 'posix-backslash.generated.js'),
  `import { createRequire as _createRequire } from "module";

it("should create require from absolute file URL object", () => {
\texpect(_createRequire(new URL(${JSON.stringify(pathToFileURL(path.join(__dirname, 'foo/c.js')).href)}))("./a")).toBe(4);
});

it("should create require from absolute file URL object with ignored base", () => {
\texpect(_createRequire(new URL(${JSON.stringify(pathToFileURL(path.join(__dirname, 'foo/c.js')).href)}, undefined))("./a")).toBe(4);
});

it("should normalize direct file URL dot segments", () => {
\texpect(_createRequire(${JSON.stringify(`${pathToFileURL(`${__dirname}${path.sep}`).href}foo/..`)})("./a")).toBe(1);
});

it("should accept normalized file URL object spellings", () => {
\texpect(_createRequire(new URL(${JSON.stringify(pathToFileURL(path.join(__dirname, 'foo/c.js')).href.replace('file:///', 'file:/'))}, import.meta.url))("./a")).toBe(4);
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

/** @type {import("@rspack/core").Configuration} */
module.exports = {
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
};
