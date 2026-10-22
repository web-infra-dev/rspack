import path from "node:path";
import { createProdNormalCase, describeByWalk } from "@rspack/test-tools";

describeByWalk(import.meta.filename, (name, src, dist) => {
	createProdNormalCase(name, src, dist);
}, {
	source: path.resolve(import.meta.dirname, "./normalCases"),
	dist: path.resolve(import.meta.dirname, `./js/normal-prod`),
});
