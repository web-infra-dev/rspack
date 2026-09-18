import path from "node:path";
import { describeByWalk, createHotStepCase } from "@rspack/test-tools";
const tempDir = path.resolve(import.meta.dirname, `./js/temp/hot-snapshot`);

describeByWalk(import.meta.filename, (name, src, dist) => {
	createHotStepCase(name, src, dist, path.join(tempDir, name), "web");
}, {
	source: path.resolve(import.meta.dirname, "./hotCases"),
	dist: path.resolve(import.meta.dirname, `./js/hot-snapshot`),
	exclude: [/remove-add-worker/]
});
