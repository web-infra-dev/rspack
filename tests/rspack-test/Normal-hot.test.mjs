import path from "node:path";
import { createHotNormalCase, describeByWalk } from "@rspack/test-tools";

describeByWalk(import.meta.filename, (name, src, dist) => {
	createHotNormalCase(name, src, dist);
}, {
	source: path.resolve(import.meta.dirname, "./normalCases"),
	dist: path.resolve(import.meta.dirname, `./js/normal-hot`),
	exclude: [
		/esm-commonjs-mix-decorator/
	]
});
