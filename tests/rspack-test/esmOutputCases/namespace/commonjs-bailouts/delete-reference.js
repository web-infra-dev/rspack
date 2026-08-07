"use strict";

exports.value = 1;
exports.read = function read() {
	return exports.value;
};
exports.remove = function remove() {
	return delete exports.value;
};
