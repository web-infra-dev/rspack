const { describeByWalk, createDiagnosticCase } = require("@rspack/test-tools");

describeByWalk(__filename, (name, src, dist) => {
	createDiagnosticCase(name, src, dist, {
		absoluteDist: false
	});
});
