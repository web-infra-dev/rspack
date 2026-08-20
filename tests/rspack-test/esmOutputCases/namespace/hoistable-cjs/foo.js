"use strict";

const __RSPACK_CJS_EXPORT_value__ = 41;
var __rspack_unused_export = 42;
globalThis.__RSPACK_CJS_EXPORT_readGlobal__ = 43;
globalThis.__RSPACK_CJS_EXPORT_defined__ = 44;

exports.unused = "unused";
exports.value = 1;
exports.local = __RSPACK_CJS_EXPORT_value__;
exports.placeholder = __rspack_unused_export;
exports.setValue = function setValue(value) {
  exports.value = value;
};
exports.getValue = function getValue() {
  return exports.value;
};
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
exports.anonymousFunction = function () {};
exports.anonymousArrow = () => {};
exports.AnonymousClass = class {};
exports.defined = 1;
exports.definedAnonymous = __CJS_ANONYMOUS_DEFINE__;
exports.readDefined = () => {
  const value = __CJS_COLLISION_DEFINE__;
  delete globalThis.__RSPACK_CJS_EXPORT_defined__;
  return value;
};
exports["a-b"] = "a-b-value";
exports.a_b_612d62 = "identifier-value";
exports.chainedA = exports.chainedB = "chained-value";
exports.__rspack_cjs_external_setter__ = 45;
exports.externalSetterObserved =
	globalThis.__rspack_cjs_external_setter_seen__;
