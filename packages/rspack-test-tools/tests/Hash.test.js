const { createHashCase, describeByWalk } = require("../dist");

describeByWalk(__filename, (name, src, dist) => {
	createHashCase(name, src, dist);
}, {
	level: 1,
	absoluteDist: false
});
