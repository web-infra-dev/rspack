const { describeByWalk, createBuiltinCase } = require("..");

describeByWalk(__filename, (name, src, dist) => {
	createBuiltinCase(name, src, dist);
});
