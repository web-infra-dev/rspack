/*
	MIT License http://www.opensource.org/licenses/mit-license.php
	Author Tobias Koppers @sokra
*/

"use strict";

// /** @typedef {import("./ChunkGraph")} ChunkGraph */
// /** @typedef {import("./Module")} Module */
// /** @typedef {import("./RequestShortener")} RequestShortener */
// /** @typedef {typeof import("./util/Hash")} Hash */
/** @typedef {any} ChunkGraph */
/** @typedef {any} Module */
/** @typedef {any} RequestShortener */
/** @typedef {any} Hash */

const ModuleFilenameHelpers = exports;

// @ts-expect-error
const asRegExp = test => {
	if (typeof test === "string") {
		test = new RegExp("^" + test.replace(/[-[\]{}()*+?.,\\^$|#\s]/g, "\\$&"));
	}
	return test;
};

// @ts-expect-error
ModuleFilenameHelpers.matchPart = (str, test) => {
	if (!test) return true;
	test = asRegExp(test);
	if (Array.isArray(test)) {
		return test.map(asRegExp).some(regExp => regExp.test(str));
	} else {
		return test.test(str);
	}
};

// @ts-expect-error
ModuleFilenameHelpers.matchObject = (obj, str) => {
	if (obj.test) {
		if (!ModuleFilenameHelpers.matchPart(str, obj.test)) {
			return false;
		}
	}
	if (obj.include) {
		if (!ModuleFilenameHelpers.matchPart(str, obj.include)) {
			return false;
		}
	}
	if (obj.exclude) {
		if (ModuleFilenameHelpers.matchPart(str, obj.exclude)) {
			return false;
		}
	}
	return true;
};
