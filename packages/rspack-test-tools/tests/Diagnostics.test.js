const { describeByWalk, createDiagnosticCase } = require("..");

describeByWalk(__filename, (name, src, dist) => {
	createDiagnosticCase(name, src, dist, {
		absoluteDist: false,
	});
});
