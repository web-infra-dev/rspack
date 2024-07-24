const path = require("path");

const webpack = require("webpack");
//eslint-disable-next-line node/no-unpublished-require
const [devServerVersion] = require("webpack-dev-server/package.json").version;
const [webpackVersion] = webpack.version;

const snapshotExtension = `.snap.webpack${webpackVersion}`;
const snapshotExtensionForServe = `.snap.devServer${devServerVersion}.webpack${webpackVersion}`;

const helpCommandTestDir = path.resolve(__dirname, "../test/help");
const serveCommandTestDir = path.resolve(__dirname, "../test/serve");

module.exports = {
  resolveSnapshotPath: (testPath) => {
    if (testPath.startsWith(helpCommandTestDir) || testPath.startsWith(serveCommandTestDir)) {
      return path.join(
        path.dirname(testPath),
        "__snapshots__",
        `${path.basename(testPath)}${snapshotExtensionForServe}`,
      );
    }

    return path.join(
      path.dirname(testPath),
      "__snapshots__",
      `${path.basename(testPath)}${snapshotExtension}`,
    );
  },
  resolveTestPath: (snapshotPath) =>
    snapshotPath.replace(`${path.sep}__snapshots__`, "").slice(0, -snapshotExtension.length),
  testPathForConsistencyCheck: path.join("consistency_check", "__tests__", "example.test.js"),
};
