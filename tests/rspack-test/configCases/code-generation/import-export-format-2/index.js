import { foo as cjsexport_harmonyimport } from "./cjs-module.js";
import theDefault, { bar as harmonyexport_harmonyimport } from "./harmony-module.js";
import theDefaultExpression from "./export-default-expression.js";
const { harmonyexport_cjsimport } = require("./harmony-module.js").bar;
const harmonyexport_cjsimportdefault = require("./export-default-expression.js").default;
import { baz as harmonyexport_harmonyimport_2 } from "./harmony-module-2.js";

import * as mod3 from "./harmony-module-3.js";
export { mod3 };
export { theDefaultExpression }

const { expectSourceToContain, expectSourceToMatch } = require("@rspack/test-tools/helper/legacy/expectSource");
const regexEscape = require("@rspack/test-tools/helper/legacy/regexEscape");

// It's important to use propertyName when generating object members to ensure that the exported property name
// uses the same accessor syntax (quotes vs. dot notatation) as the imported property name on the other end
// (which needs to use propertyAccess).  Else, minifiers such as Closure Compiler will not be able to minify correctly.
it("should use the same accessor syntax for import and export", function () {

	var fs = require("fs");
	var source = fs.readFileSync(__filename, "utf-8").toString();

	// Reference these imports to generate uses in the source.

	cjsexport_harmonyimport;
	harmonyexport_harmonyimport;
	harmonyexport_cjsimport;
	harmonyexport_harmonyimport_2;
	theDefault;
	theDefaultExpression;
	harmonyexport_cjsimportdefault;

	/*********** DO NOT MATCH BELOW THIS LINE ***********/

	// Checking harmonyexportinitfragment.js formation of standard export fragment
	expectSourceToContain(source, "bar: () => (bar)");

	// Checking formation of imports
	expectSourceToMatch(source, `${regexEscape("const { harmonyexport_cjsimport } = (__webpack_require__(")}\\d+${regexEscape(")/* .bar */.bar);")}`);
	expectSourceToMatch(source, `${regexEscape("const harmonyexport_cjsimportdefault = (__webpack_require__(")}\\d+${regexEscape(")/* [\"default\"] */[\"default\"]);")}`);

	// Checking concatenatedmodule.js formation of exports
	expectSourceToContain(source, "mod3: () => (/* reexport */ harmony_module_3_namespaceObject)");

	// Checking concatenatedmodule.js formation of namespace objects
	expectSourceToContain(source, "apple: () => (apple)");

	// Do not break default option
	expectSourceToContain(source, "[\"default\"] = (___CSS_LOADER_EXPORT___)");
});
