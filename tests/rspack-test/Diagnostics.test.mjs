import { describeByWalk, createDiagnosticCase } from "@rspack/test-tools";

describeByWalk(import.meta.filename, (name, src, dist) => {
	createDiagnosticCase(name, src, dist, {
		absoluteDist: false
	});
});
