import { foo as cjsexport_harmonyimport } from "./cjs-module.js";
import theDefault, { bar as harmonyexport_harmonyimport } from "./harmony-module.js";
const { harmonyexport_cjsimport } = require("./harmony-module.js").bar;
import { baz as harmonyexport_harmonyimport_2 } from "./harmony-module-2.js";

import * as mod3 from "./harmony-module-3.js";
export { mod3 };

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

	/*********** DO NOT MATCH BELOW THIS LINE ***********/

	// Note that there are no quotes around the "a" and "b" properties in the following lines.

	// Checking harmonyexportinitfragment.js formation of standard export fragment
	// DIFF:
	// expectSourceToContain(source, "/* harmony export */   a: () => (/* binding */ bar)");
	expectSourceToContain(source, "a: () => (bar)");

	// Checking formation of imports
	expectSourceToContain(source, "harmony_module/* .bar */.a;");
	// DIFF:
	// expectSourceToMatch(source, `${regexEscape("const { harmonyexport_cjsimport } = (__webpack_require__(/*! ./harmony-module */ ")}\\d+${regexEscape(")/* .bar */ .a);")}`);
	expectSourceToMatch(source, `${regexEscape("const { harmonyexport_cjsimport } = (__webpack_require__(")}\\d+${regexEscape(")/* .bar */.a);")}`);

	// Checking concatenatedmodule.js formation of exports
	expectSourceToContain(source, "a: () => (/* reexport */ harmony_module_3_namespaceObject)");

	// Checking concatenatedmodule.js formation of namespace objects
	expectSourceToContain(source, "a: () => (apple)");
});
