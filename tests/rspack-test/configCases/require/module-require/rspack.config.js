const fs = require('fs');
const path = require('path');

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
  process.platform === 'win32'
    ? '\n'
    : `import { createRequire as _createRequire } from "module";

it("should treat POSIX absolute paths ending in backslash as files", () => {
\texpect(_createRequire(__dirname + "/foo\\\\")("./posix-backslash")).toBe(
\t\t"posix-backslash"
\t);
});
`,
);

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'node',
  optimization: {
    inlineExports: true,
    moduleIds: 'named',
  },
};
