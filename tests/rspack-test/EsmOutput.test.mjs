import { describeByWalk, createEsmOutputCase } from "@rspack/test-tools";

describeByWalk(import.meta.filename, (name, src, dist) => {
	createEsmOutputCase(name, src, dist);
});
