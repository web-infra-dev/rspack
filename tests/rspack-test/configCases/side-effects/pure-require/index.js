import { value } from "./pure";
import declared from "./declared";
import { value as transitive } from "./transitive";
import { value as forced } from "./forced";
import fs from "fs";

function rawRequire() {
	return require("./declared");
}

function rawMember() {
	return require("./declared").value;
}

it("should annotate pure module initializers only for module output", () => {
	expect(value).toBe(42);
	expect(declared.value).toBe(43);
	expect(transitive).toBe(45);
	expect(forced).toBe(46);
	expect(rawRequire().value).toBe(43);
	expect(rawMember()).toBe(43);
	expect(rawRequire.toString()).not.toContain("/*#__PURE__*/");
	expect(rawMember.toString()).not.toContain("/*#__PURE__*/");

	const source = fs.readFileSync(__filename, "utf-8");
	const runtime = globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK
		? "__rspack_context.r"
		: "__webpack_require__";
	const pureLoad = request =>
		["= /*#__PURE__*/ ", runtime, '("./', request, '.js")'].join("");
	expect(source.includes(pureLoad("declared"))).toBe(MODULE_OUTPUT);
	if (!CONCATENATED) {
		expect(source.includes(pureLoad("pure"))).toBe(MODULE_OUTPUT && ANALYZED);
	}
	expect(source).not.toContain(pureLoad("transitive"));
	expect(source).not.toContain(pureLoad("forced"));
});
