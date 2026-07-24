"use strict";

exports.value = 1;
exports.setValue = function setValue(value) {
  exports.value = value;
};
exports.getValue = function getValue() {
  return exports.value;
};
exports.unused = "unused";
