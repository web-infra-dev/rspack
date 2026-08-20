var _a;

export const value =
	process.env.INLINED_VALUE === "inlined" && typeof process !== "undefined"
		? (_a = process.env) === null || _a === void 0
			? void 0
			: _a.PROVIDED_VALUE
		: "missing";
