it("should evaluate basic constant expressions like webpack", () => {
	if (String(42) !== "42") require("./non-existent-module");
	if (Number("5") * 2 !== 10) require("./non-existent-module");
	if (Boolean(0) !== false) require("./non-existent-module");
	if ("abcdef".slice(1, 3) !== "bc") require("./non-existent-module");
	if ("abcdef".substr(2, 3) !== "cde") require("./non-existent-module");
	if ("abcdef".substring(1, 4) !== "bcd") require("./non-existent-module");
});

it("should evaluate bigint and arithmetic operators", () => {
	// BigInt addition
	if (1n + 2n !== 3n) require("./non-existent-module");

	// Numeric Mod
	if (5 % 2 !== 1) require("./non-existent-module");
	if (10 % 4 !== 2) require("./non-existent-module");

	// Unsigned right shift
	if (-1 >>> 0 !== 0xffffffff) require("./non-existent-module");
	if (1 >>> 0 !== 1) require("./non-existent-module");
});

it("should evaluate typeof and keep side effects for wrapped expressions", () => {
	let called = 0;
	function sideEffect() {
		called++;
		return "x";
	}

	if (typeof +1 !== "number") require("./non-existent-module");
	if (typeof ("a" + sideEffect()) !== "string") require("./non-existent-module");
	if (called !== 1) {
		throw new Error("sideEffect should be called exactly once");
	}
});

