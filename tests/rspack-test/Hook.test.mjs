import path from "node:path";
import { createHookCase, describeByWalk } from "@rspack/test-tools";
const source = path.resolve(import.meta.dirname, "./fixtures");

describeByWalk(import.meta.filename, (name, src, dist) => {
	createHookCase(name, src, dist, source);
});
