const { describeByWalk, createExampleCase } = require("@rspack/test-tools");

describeByWalk(__filename, (name, src, dist) => {
  createExampleCase(name, src, dist);
}, {
  level: 1,
});
