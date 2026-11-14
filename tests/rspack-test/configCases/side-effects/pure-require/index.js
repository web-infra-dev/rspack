import { value } from "./pure";
import declared from "./declared";
import { value as transitive } from "./transitive";
import { value as forced } from "./forced";

function loadDeclared() {
	return require("./declared");
}

function loadConditionalPure(flag) {
	return require(flag ? "./declared" : "./pure");
}

function loadConditionalMixed(flag) {
	return require(flag ? "./declared" : "./transitive");
}

function unusedPureRequire() {
	require("./declared");
}

it("should annotate only side-effect-free module loads", () => {
	expect(value).toBe(42);
	expect(declared.value).toBe(43);
	expect(transitive).toBe(45);
	expect(forced).toBe(46);
	expect(loadDeclared().value).toBe(43);
	expect(loadConditionalPure(true).value).toBe(43);
	expect(loadConditionalPure(false).value).toBe(42);
	expect(loadConditionalMixed(false).value).toBe(45);

	if (!MINIMIZED) {
		const source = require("fs").readFileSync(__filename, "utf-8");
		const runtime = globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK
			? "__rspack_context.r"
			: "__webpack_require__";
		const pureImport = request =>
			["= (0, /*#__PURE__*/ ", runtime, '("./', request, '.js"))'].join("");
		expect(source).toContain(pureImport("declared"));
		if (ANALYZED) {
			expect(source).toContain(pureImport("pure"));
		} else {
			expect(source).not.toContain(pureImport("pure"));
		}
		expect(source).not.toContain(pureImport("transitive"));
		expect(source).not.toContain(pureImport("forced"));
		expect(loadDeclared.toString()).toContain("/*#__PURE__*/");
		expect(loadConditionalPure.toString().includes("/*#__PURE__*/")).toBe(ANALYZED);
		expect(loadConditionalMixed.toString()).not.toContain("/*#__PURE__*/");
	} else {
		expect(unusedPureRequire.toString()).not.toContain("./declared.js");
	}
	unusedPureRequire();
});

it("should preserve calls to exports of pure modules when minimizing", () => {
	globalThis.pureRequireCalls = 0;
	require("./declared")();
	require("./declared").effect();
	new (require("./declared"))();
	expect(globalThis.pureRequireCalls).toBe(3);
	delete globalThis.pureRequireCalls;
});

// Missing semicolons are intentional: PURE annotations introduce a leading `(`.
it("should preserve ASI before static and conditional requires", () => {
	const calls = [];
	function before() {
		calls.push("before");
	}
	before()
	require("./declared")
	for (const flag of [true, false]) {
		before()
		require(flag ? "./declared" : "./pure")
	}
	expect(calls).toEqual(["before", "before", "before"]);
});

it("should preserve ASI before require member access and calls", () => {
	let calls = 0;
	function before() {
		calls++;
	}
	globalThis.pureRequireCalls = 0;
	before()
	require("./declared").value
	before()
	require("./declared").effect()
	before()
	require("./declared")()
	before()
	new (require("./declared"))()
	if (calls < 0) require("./declared").effect();
	expect(calls).toBe(4);
	expect(globalThis.pureRequireCalls).toBe(3);
	delete globalThis.pureRequireCalls;
});
