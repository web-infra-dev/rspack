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

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'node',
  optimization: {
    inlineExports: true,
    moduleIds: 'named',
  },
};
