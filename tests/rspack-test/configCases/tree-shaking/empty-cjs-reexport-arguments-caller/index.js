import "./empty";
import "./mutator";
import * as barrel from "./barrel";

const findModule = name => __STATS__.modules.find(m => m.name.endsWith(`/${name}`));

it("should keep empty exports unknown when factory arguments expose runtime require", () => {
	expect(globalThis.argumentsCallerTargetLoaded).toBe(true);
	expect(barrel.argumentsCallerValue).toBe("arguments caller");
	expect(barrel.argumentsValue).toBe("arguments");
	expect(barrel.arrowArgumentsValue).toBe("arrow arguments");
	expect(barrel.varArgumentsValue).toBe("var arguments");
	expect(findModule("empty.js").providedExports).toBe(null);
	expect(findModule("access-arguments.js").providedExports).toBe(null);
	expect(findModule("access-arrow-arguments.js").providedExports).toBe(null);
	expect(findModule("access-var-arguments.js").providedExports).toBe(null);
});
