import "./calls";

const required = [
	require("./functions"),
	require("./default-function"),
	require("./default-arrow"),
	require("./default-identifier"),
	require("./default-this"),
	require("./eval-module")
];

const source = require("fs").readFileSync(__filename, "utf-8");
const access = (module, name) =>
	`[\\w$]*${module}[\\w$]*(?:\\.${name}|\\[${JSON.stringify(name).replace(/"/g, '\\"')}\\])`;
const bareCall = (module, name) =>
	new RegExp(`(?<![\\w$,])${access(module, name)}\\(`);
const wrappedCall = (module, name) =>
	new RegExp(`\\(0,\\s*${access(module, name)}\\)\\(`);

it("should drop the indirect call wrapper only for this-free exports", () => {
	expect(required.length).toBe(6);
	for (const [module, name] of [
		["functions", "plainFunction"],
		["functions", "arrowFunction"],
		["functions", "functionExpression"],
		["functions", "nestedFunctionThis"],
		["functions", "nestedClassThis"],
		["functions", "renamedLater"],
		["default_function", "default"],
		["default_arrow", "default"],
		["default_identifier", "default"]
	]) {
		expect(source).toMatch(bareCall(module, name));
		expect(source).not.toMatch(wrappedCall(module, name));
	}
	for (const [module, name] of [
		["functions", "usesThis"],
		["functions", "arrowThis"],
		["functions", "parameterThis"],
		["functions", "computedKeyThis"],
		["functions", "reassigned"],
		["default_this", "default"],
		["eval_module", "evalNeighbor"]
	]) {
		expect(source).toMatch(wrappedCall(module, name));
		expect(source).not.toMatch(bareCall(module, name));
	}
});
