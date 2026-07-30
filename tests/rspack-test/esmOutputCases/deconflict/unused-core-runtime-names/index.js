import { publicPath, value } from "./value.js";
import { readGlobalModuleCache } from "./global.js";

function rspackRequire() {}
function modules() {}
function moduleCache() {}

export const actualNames = [
	rspackRequire.name,
	publicPath.name,
	modules.name,
];

it("should not reserve core runtime names when no runtime is emitted", () => {
	expect(actualNames).toEqual(["rspackRequire", "publicPath", "modules"]);
	expect(value).toBe(42);

	const globalModuleCache = readGlobalModuleCache();
	delete globalThis.moduleCache;
	expect(globalModuleCache).toBe("global");
	expect(moduleCache.name).not.toBe("moduleCache");
});
