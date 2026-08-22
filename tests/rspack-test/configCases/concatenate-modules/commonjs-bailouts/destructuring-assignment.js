"use strict";

exports.value = 1;
exports.assign = function assign() {
	({ value: module.exports.value } = { value: 2 });
};
