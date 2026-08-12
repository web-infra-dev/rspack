"use strict";

const __RSPACK_CJS_EXPORT_value__ = 41;
var __rspack_unused_export = 42;

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
