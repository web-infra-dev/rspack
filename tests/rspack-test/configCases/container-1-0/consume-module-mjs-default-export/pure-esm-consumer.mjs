import something from "./shared-esm-pkg/index.js";
import { namedExport } from "./shared-esm-pkg/index.js";

export function testDefaultImport() {
	return {
		defaultType: typeof something,
		defaultValue: typeof something === "function" ? something() : something,
		namedExportValue: namedExport
	};
}
