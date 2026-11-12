import fs from "node:fs";
import { checkChunkModules } from "@rspack/test-tools";

// Changing the outer block's location would change its identifier and mask
// the reuse bug. Step 1 replaces only the nested ensure with equal-length code.
const original = fs.readFileSync(new URL("./0/index.js", import.meta.url), "utf8");
const removed = fs.readFileSync(new URL("./1/index.js", import.meta.url), "utf8");
expect(removed.split("\n").map(line => line.length)).toEqual(
	original.split("\n").map(line => line.length)
);

export default {
	checkStats(step, stats) {
		const nested = step !== "1";
		expect(stats.chunks.flatMap(chunk => chunk.names).sort()).toEqual(
			(nested ? ["inner", "main", "other", "outer"] : ["main", "other", "outer"])
		);
		expect(stats.assets.some(asset => asset.name === "inner.js")).toBe(nested);
		checkChunkModules(stats, {
			outer: ["a.js"],
			other: ["b.js", "c.js"],
			...(nested ? { inner: ["b.js"] } : {})
		});
		return true;
	}
};
