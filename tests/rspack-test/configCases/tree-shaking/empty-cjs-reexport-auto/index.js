export * from "./barrel";

import { amdFactory, amdObject } from "./amd-barrel";
import { arrowThisValue } from "./arrow-barrel";
import { webpackModuleValue } from "./webpack-module-barrel";
import { commonJsValue } from "./disabled-commonjs-barrel";
import "./mutate-external-require";
import "./mutate-external-import";
import { importValue, requireValue } from "./external-barrel";
import { noParseValue } from "./no-parse-barrel";
import "./side-effect-import";
import "./access-exports";
import "./access-module";
import "./access-this";

const findModule = name => __STATS__.modules.find(m => m.name.endsWith(`/${name}`));

it("should infer empty exports for auto modules without cjs export access", () => {
	expect(globalThis.emptySideEffectImportRan).toBe(true);
	expect(findModule("index.js").providedExports).toEqual(["live"]);
});

it("should keep unknown exports when cjs export access is observed", () => {
	expect(arrowThisValue).toBe("arrow this");
	expect(webpackModuleValue).toBe("webpack module");
	expect(findModule("access-arrow-this.js").providedExports).toBe(null);
	expect(findModule("access-webpack-module.js").providedExports).toBe(null);
	expect(findModule("access-exports.js").providedExports).toBe(null);
	expect(findModule("access-module.js").providedExports).toBe(null);
	expect(findModule("access-this.js").providedExports).toBe(null);
});

it("should keep amd define exports unknown", () => {
	expect(amdFactory).toBe("factory");
	expect(amdObject).toBe("object");
	expect(findModule("amd-factory.js").providedExports).toBe(null);
	expect(findModule("amd-object.js").providedExports).toBe(null);
});

it("should keep exports unknown when commonjs export parsing is disabled", () => {
	expect(commonJsValue).toBe("commonjs");
	expect(findModule("disabled-commonjs.js").providedExports).toBe(null);
});

it("should keep exports unknown when another module can mutate them", () => {
	expect(requireValue).toBe("require");
	expect(importValue).toBe("import");
	expect(findModule("external-empty.js").providedExports).toBe(null);
});

it("should keep exports unknown when the module was not parsed", () => {
	expect(noParseValue).toBe("no-parse");
	expect(findModule("no-parse.js").providedExports).toBe(null);
});
