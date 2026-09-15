import path from "node:path";
import { createDevNormalCase, describeByWalk } from "@rspack/test-tools";

describeByWalk(import.meta.filename, (name, src, dist) => {
	createDevNormalCase(name, src, dist);
}, {
	source: path.resolve(import.meta.dirname, "./normalCases"),
	dist: path.resolve(import.meta.dirname, `./js/normal-dev`),
});
