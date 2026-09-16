const path = require('node:path');
const { pathToFileURL } = require('node:url');
const { DefinePlugin } = require('@rspack/core');

/** @type {import("@rspack/core").Configuration[]} */
module.exports = ['tsconfig.json', 'tsconfig-no-base-url.json'].flatMap(
  (filename) => {
    const configFile = path.join(__dirname, filename);
    return [configFile, path.relative(process.cwd(), configFile)].flatMap(
      (configFile) =>
        [configFile, { configFile }].map((tsConfig) => ({
          resolve: { tsConfig },
          plugins: [
            new DefinePlugin({
              EXPECTED_URL: JSON.stringify(
                pathToFileURL(path.join(__dirname, 'generated/client.js')).href,
              ),
            }),
          ],
        })),
    );
  },
);
