import { effectValue, pureValue } from "./a";

const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

const state =
	globalThis.__configCases_sideEffects_reexportPreservedEvalChainPartialRetarget ??
	(globalThis.__configCases_sideEffects_reexportPreservedEvalChainPartialRetarget = {
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
	expect(pureValue()).toBe("d");
	expect(effectValue()).toBe("e");
	expect(state.a).toBe("a");
	expect(state.e).toBe("e");
	expect(state.order).toEqual(["e", "a", "index"]);
});

it("should retarget only the side-effect-free binding", () => {
	const indexBlock = getModuleBlock(generated, "./index.js");
	const aModuleId = getModuleId(generated, "./a.js");
	const dModuleId = getModuleId(generated, "./d.js");
	const eModuleId = getModuleId(generated, "./e.js");

	expect(indexBlock).toContain(`__webpack_require__(/*! ./a */ ${aModuleId})`);
	expect(indexBlock).toContain(`__webpack_require__(/*! ./a */ ${dModuleId})`);
	expect(indexBlock).not.toContain(`__webpack_require__(/*! ./a */ ${eModuleId})`);
	expect(generated).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, "bundle0.js.txt"));
});

afterAll(() => {
	delete globalThis.__configCases_sideEffects_reexportPreservedEvalChainPartialRetarget;
});
