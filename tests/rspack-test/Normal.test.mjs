import { createNormalCase, describeByWalk } from "@rspack/test-tools";

describeByWalk(import.meta.filename, (name, src, dist) => {
	createNormalCase(name, src, dist);
});
