"use strict";

globalThis.__RSPACK_CJS_EXPORT_defined__ = 96;
globalThis.__RSPACK_CJS_EXPORT_escapedDefined__ = 95;

exports.defined = 1;
exports.definedAnonymous = __CJS_ANONYMOUS_DEFINE__;
exports.readDefined = () => {
	const value = __CJS_COLLISION_DEFINE__;
	delete globalThis.__RSPACK_CJS_EXPORT_defined__;
	return value;
};
exports.escapedDefined = 1;
exports.readEscapedDefined = () => {
	const value = __CJS_ESCAPED_COLLISION_DEFINE__;
	delete globalThis.__RSPACK_CJS_EXPORT_escapedDefined__;
	return value;
};
