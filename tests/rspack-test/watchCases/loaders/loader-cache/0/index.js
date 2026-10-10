import { E } from "./enum";

const value = require("./value");
const moduleA = require("./module-a");
const moduleB = require("./module-b");
const bom = require("./bom");
const fileDependency = require("./file-dependency");
const contextDependency = require("./context-dependency");
const buildDependency = require("./build-dependency");
const missingDependency = require("./missing-dependency");
const chainDependency = require("./chain-dependency");
const overlapDependency = require("./overlap-dependency");
const jsOverlapDependency = require("./js-overlap-dependency");

// Cached loaders run initially, then run again when their input content
// changes at steps 2 and 4.
const CACHED_LOADER_RUNS_BY_STEP = [1, 1, 2, 2, 3];
// Changing the dependencies inherited from a preceding loader invalidates the
// etag at step 1. Changing this loader's own file dependency still invalidates
// its stored mtime snapshot at step 2.
const OVERLAP_LOADER_RUNS_BY_STEP = [1, 2, 3, 3, 3];
// Adjacent cached loaders form one cache chain, so changing the right loader's
// dependency at step 2 invalidates and reruns the whole chain.
const CACHE_CHAIN_RUNS_BY_STEP = [1, 1, 2, 2, 2];

it("should cache each opted-in loader until its input changes", () => {
	const step = +WATCH_STEP;
	const cachedRuns = CACHED_LOADER_RUNS_BY_STEP[step];
	expect(value).toEqual({
		value: step < 2 ? "initial" : step < 4 ? "changed-2" : "changed-4",
		leftRuns: step + 1,
		markedRuns: cachedRuns,
		rightRuns: cachedRuns,
		sourceMap: true
	});
	expect(moduleA).toBe("module-a.js");
	expect(moduleB).toBe("module-b.js");
	expect(E.A).toBe(0);
	const generated = require("fs").readFileSync(__filename, "utf-8");
	expect(generated).toContain("inlined export .E.A");
	expect(bom).toEqual({
		hasBom: true,
		producerRuns: 1,
		consumerRuns: step + 1
	});
	expect(fileDependency).toEqual({
		value: step < 2 ? "file-0" : step < 4 ? "file-2" : "file-4",
		runs: cachedRuns
	});
	expect(contextDependency).toEqual({
		value: step < 2 ? ["0.txt"] : step < 4 ? ["0.txt", "2.txt"] : ["0.txt", "2.txt", "4.txt"],
		// Context dependencies are intentionally unsupported by the minimal loader cache.
		runs: step + 1,
		downstreamRuns: step + 1
	});
	expect(buildDependency).toEqual({
		value: step < 2 ? "build-0" : step < 4 ? "build-2" : "build-4",
		runs: cachedRuns
	});
	expect(missingDependency).toEqual({
		value: `trigger-${step}`,
		runs: step + 1
	});
	expect(chainDependency).toEqual({
		leftRuns: CACHE_CHAIN_RUNS_BY_STEP[step]
	});
	expect(overlapDependency).toEqual({
		value: step < 2 ? "overlap-0" : "overlap-2",
		runs: OVERLAP_LOADER_RUNS_BY_STEP[step]
	});
	expect(jsOverlapDependency).toEqual({
		value: step < 2 ? "overlap-0" : "overlap-2",
		runs: OVERLAP_LOADER_RUNS_BY_STEP[step]
	});
});
