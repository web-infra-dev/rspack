import { E } from "./enum";
import "./style.css";

const value = require("./value");
const moduleA = require("./module-a");
const moduleB = require("./module-b");

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
		sourceMapConsumerRuns: step + 1,
		sourceMapInput: `value-${step + 1}.js`,
		sourceMap: true
	});
	expect(moduleA).toBe("module-a.js");
	expect(moduleB).toBe("module-b.js");
	expect(E.A).toBe(0);
	const generated = require("fs").readFileSync(__filename, "utf-8");
	expect(generated).toContain("inlined export .E.A");
	const css = require("fs").readFileSync(
		require("path").join(__dirname, "main.css"),
		"utf-8"
	);
	expect(css).toContain("color: red");
});
