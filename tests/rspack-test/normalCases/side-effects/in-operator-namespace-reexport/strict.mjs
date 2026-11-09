import * as ns from "./reexport.js";
import * as defaultNs from "./default-reexport.js";
import * as chainNs from "./chain.js";
import * as cjsRequire from "cjs-require-lib";

export const strictResults = {
	inner: "inner" in ns,
	cjsInner: "cjsInner" in ns,
	flaggedInner: "flaggedInner" in ns,
	jsonInner: "jsonInner" in ns,
	effectInner: "effectInner" in ns,
	dynamicInner: "dynamicInner" in ns,
	fnInner: "fnInner" in ns,
	default: "default" in defaultNs,
	chained: "inner" in chainNs,
	cjsRequire: "inner" in cjsRequire
};
