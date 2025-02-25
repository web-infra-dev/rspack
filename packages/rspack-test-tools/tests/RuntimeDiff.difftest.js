const { createDiffCase, describeByWalk } = require("..");
const path = require("path");

jest.disableAutomock();

describeByWalk(__filename, (name, src, dist) => {
	createDiffCase(name, src, dist);
}, {
	dist: path.resolve(__dirname, `./js/runtime-diff`)
});
