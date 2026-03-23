import { value } from "./a";

const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

const state =
	globalThis.__configCases_sideEffects_reexportPreservedEvalChainRetarget ??
	(globalThis.__configCases_sideEffects_reexportPreservedEvalChainRetarget = {
		order: []
	});
const bundlePath = path.resolve(__dirname, "bundle0.js");
const generated = fs.readFileSync(bundlePath, "utf-8");

state.order.push("index");

function getModuleBlock(source, moduleId) {
	const start = source.indexOf(`*** ${moduleId} ***`);
	expect(start).toBeGreaterThan(-1);
	const end = source.indexOf('\nconst fs = require("fs");', start);
	expect(end).toBeGreaterThan(start);
	return source.slice(start, end);
}

function getModuleId(source, moduleId) {
	const start = source.indexOf(`*** ${moduleId} ***`);
	expect(start).toBeGreaterThan(-1);
	const lines = source.slice(0, start).split("\n");

	for (let i = lines.length - 1; i >= 0; i -= 1) {
		const line = lines[i].trim();
		if (/^\d+$/.test(line)) {
			return line;
		}
	}

	throw new Error(`Cannot find module id for ${moduleId}`);
}

function escapeRegExp(value) {
	return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function hasModuleDefinition(source, moduleId) {
	return new RegExp(
		String.raw`\n\d+\n/\*![\s\S]*?\n\s+\*{3} ${escapeRegExp(moduleId)} \*{3}!`,
		"m"
	).test(source);
}

it("should preserve a side effects and execution order", () => {
	expect(value()).toBe("d");
	expect(state.a).toBe("a");
	expect(state.order).toEqual(["a", "index"]);
});

it("should retarget the binding through side-effect-free links", () => {
	const indexBlock = getModuleBlock(generated, "./index.js");
	const aModuleId = getModuleId(generated, "./a.js");
	const dModuleId = getModuleId(generated, "./d.js");

	expect(indexBlock).toContain(`__webpack_require__(/*! ./a */ ${aModuleId})`);
	expect(indexBlock).toContain(`__webpack_require__(/*! ./a */ ${dModuleId})`);
	expect(hasModuleDefinition(generated, "./b.js")).toBe(false);
	expect(hasModuleDefinition(generated, "./c.js")).toBe(false);
	expect(generated).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, "bundle0.js.txt"));
});

afterAll(() => {
	delete globalThis.__configCases_sideEffects_reexportPreservedEvalChainRetarget;
});
