"use strict";

const __RSPACK_CJS_EXPORT_value__ = 99;
var __rspack_unused_export = 98;
globalThis.__RSPACK_CJS_EXPORT_readGlobal__ = 97;

exports.unused = "unused";
exports.value = 1;
exports.local = __RSPACK_CJS_EXPORT_value__;
exports.placeholder = __rspack_unused_export;
exports.readGlobal = () => __RSPACK_CJS_EXPORT_readGlobal__;
exports.nestedValue = 1;
exports.readNestedValue = () => {
	const __RSPACK_CJS_EXPORTS__ = { nestedValue: 100 };
	return exports.nestedValue;
};
exports.setNestedValue = value => {
	const __RSPACK_CJS_SET_EXPORT_nestedValue_6e657374656456616c7565__ = () => {};
	exports.nestedValue = value;
};
