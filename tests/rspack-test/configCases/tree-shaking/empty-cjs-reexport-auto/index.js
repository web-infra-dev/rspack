import * as values from "./barrel";

const findModule = name =>
	__STATS__.modules.find(module => module.name.endsWith(`/${name}`));

it("infers empty exports from CommonJS modules without export access", () => {
	expect(findModule("empty.js").providedExports).toEqual([]);
	expect(findModule("sloppy-empty.js").providedExports).toEqual([]);
});

it("keeps local CommonJS export escape hatches conservative", () => {
	expect(values.fromArguments).toBe("arguments");
	expect(values.fromArrowThis).toBe("arrow-this");
	expect(values.fromWebpackModule).toBe("webpack-module");
	for (const name of [
		"arguments.js",
		"arrow-this.js",
		"eval.js",
		"webpack-module.js",
		"exports.js",
		"module.js",
		"this.js"
	]) {
		expect(findModule(name).providedExports).toBe(null);
	}
});

it("does not infer empty exports outside eligible javascript/auto modules", () => {
	expect(values.fromAmd).toBe("amd");
	expect(values.fromDisabled).toBe("disabled");
	expect(values.fromNoParse).toBe("no-parse");
	expect(findModule("amd.js").providedExports).toBe(null);
	expect(findModule("disabled.js").providedExports).toBe(null);
	expect(findModule("dynamic.js").providedExports).toBe(null);
	expect(findModule("no-parse.js").providedExports).toBe(null);
});
