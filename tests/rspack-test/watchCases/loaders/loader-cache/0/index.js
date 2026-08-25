const value = require("./value");
const moduleA = require("./module-a");
const moduleB = require("./module-b");
const bom = require("./bom");
const fileDependency = require("./file-dependency");
const contextDependency = require("./context-dependency");
const buildDependency = require("./build-dependency");
const missingDependency = require("./missing-dependency");
const chainDependency = require("./chain-dependency");

// Cached loaders run initially, then run again when their input content
// changes at steps 2 and 4.
const CACHED_LOADER_RUNS_BY_STEP = [1, 1, 2, 2, 3];

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
		runs: cachedRuns
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
		leftRuns: 1
	});
});
