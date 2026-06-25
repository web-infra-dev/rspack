import * as valid from "./valid.module.css";
import * as nocheck from "./no-check.module.css";
import * as autoPure from "./auto-pure.module.css";
import "./auto-non-module.css";

const never = false;

if (never) {
	require("./invalid.module.css");
	require("./auto-impure.module.css");
}

it("should expose locals from a pure-compliant css module", () => {
	expect(valid).toEqual(
		nsObj({
			foo: "valid_module_css-foo",
			bar: "valid_module_css-bar",
			baz: "valid_module_css-baz",
			qux: "valid_module_css-qux",
			inner: "valid_module_css-inner",
			normal: "valid_module_css-normal"
		})
	);
});

it("should expose locals from the no-check fixture after unsupported blocks are skipped", () => {
	expect(nocheck).toEqual(
		nsObj({
			ok: "no-check_module_css-ok"
		})
	);
});

it("should apply pure check when css/auto resolves a filename to a CSS module", () => {
	expect(autoPure).toEqual(
		nsObj({
			"local-via-auto": "auto-pure_module_css-local-via-auto"
		})
	);
});
