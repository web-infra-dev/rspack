import { describeByWalk, createSerialCase } from "@rspack/test-tools";

describeByWalk(import.meta.filename, (name, src, dist) => {
	createSerialCase(name, src, dist);
});
