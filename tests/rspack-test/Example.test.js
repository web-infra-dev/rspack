const { describeByWalk, createExampleCase } = require("@rspack/test-tools");
const path = require("node:path");

describeByWalk(__filename, (name, src, dist) => {
  createExampleCase(name, src, dist);
}, {
  level: 1,
  source: path.join(__dirname, "..", "webpack-examples"),
});
