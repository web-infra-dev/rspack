import { describeByWalk, createBuiltinCase } from "@rspack/test-tools";

describeByWalk(import.meta.filename, (name, src, dist) => {
	createBuiltinCase(name, src, dist);
});
