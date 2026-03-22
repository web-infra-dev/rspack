import { b, d } from "./lib";

const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

const state =
	globalThis.__configCases_sideEffects_reexportPreservedEvalMultiSpecifier ??
	(globalThis.__configCases_sideEffects_reexportPreservedEvalMultiSpecifier = {
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

it("should preserve side effects and execution order", () => {
	expect(b()).toBe("b");
	expect(d()).toBe("d");
	expect(state.a).toBe("a");
	expect(state.c).toBe("c");
	expect(state.order).toEqual(["a", "c", "index"]);
});

it("should retarget both bindings while keeping the side-effect import", () => {
	const indexBlock = getModuleBlock(generated, "./index.js");
	const libModuleId = getModuleId(generated, "./lib.js");
	const bModuleId = getModuleId(generated, "./b.js");
	const dModuleId = getModuleId(generated, "./d.js");

	expect(indexBlock).toContain(`__webpack_require__(/*! ./lib */ ${libModuleId})`);
	expect(indexBlock).toContain(`__webpack_require__(/*! ./lib */ ${bModuleId})`);
	expect(indexBlock).toContain(`__webpack_require__(/*! ./lib */ ${dModuleId})`);
	expect(generated).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, "bundle0.js.txt"));
});

afterAll(() => {
	delete globalThis.__configCases_sideEffects_reexportPreservedEvalMultiSpecifier;
});
