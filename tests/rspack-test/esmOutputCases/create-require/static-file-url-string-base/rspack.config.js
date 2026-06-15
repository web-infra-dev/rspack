const fs = require('fs');
const path = require('path');
const { pathToFileURL } = require('url');

fs.writeFileSync(
  path.join(__dirname, 'static-base.generated.js'),
  `import { createRequire } from "module";

it("should create require from relative URL object with file URL string base in ESM output", () => {
\texpect(createRequire(new URL("./foo/c.js", ${JSON.stringify(pathToFileURL(path.join(__dirname, 'index.js')).href)}))("./a")).toBe("foo");
});
`,
);

module.exports = {
  module: {
    parser: {
      javascript: {
        createRequire: true,
      },
    },
  },
};
