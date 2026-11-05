import { createNormalCase, describeByWalk } from "@rspack/test-tools";
import { getCacheTestOptions } from "./cacheTestOptions.mjs";

describeByWalk(import.meta.filename, (name, src, dist) => {
	createNormalCase(name, src, dist, getCacheTestOptions("normal", src));
});
