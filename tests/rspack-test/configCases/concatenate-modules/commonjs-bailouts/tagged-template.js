"use strict";

exports.value = 1;
exports.tag = function tag() {
	return this.value;
};
exports.run = function run() {
	return exports.tag``;
};
