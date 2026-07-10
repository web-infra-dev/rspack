import moduleDefault, { createRequire } from "node:module";
import * as moduleNs from "node:module";

const __rspack_createRequire = "local-createRequire";
const __rspack_createRequire_require = "local-require";
const external_node_module_namespaceObject = "local-node-module";
const external_node_module_default = "local-node-module-default";
const strippedRequire = createRequire(import.meta.url);
const namedRequire = createRequire(import.meta.url);
const namespaceRequire = moduleNs.createRequire(import.meta.url);
const defaultRequire = moduleDefault.createRequire(import.meta.url);

export const helperCollision = {
	required: strippedRequire("./dep.js"),
	resolved: namedRequire.resolve("path"),
	namespaceResolved: namespaceRequire.resolve("path"),
	defaultResolved: defaultRequire.resolve("path"),
	sourceCreateRequire: __rspack_createRequire,
	sourceCreateRequireRequire: __rspack_createRequire_require,
	sourceExternalNodeModule: external_node_module_namespaceObject,
	sourceExternalNodeModuleDefault: external_node_module_default
};

it("deconflicts preserved createRequire calls with helper symbols", () => {
	expect(helperCollision).toEqual({
		required: "dep",
		resolved: "path",
		namespaceResolved: "path",
		defaultResolved: "path",
		sourceCreateRequire: "local-createRequire",
		sourceCreateRequireRequire: "local-require",
		sourceExternalNodeModule: "local-node-module",
		sourceExternalNodeModuleDefault: "local-node-module-default"
	});
});
