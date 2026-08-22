"use strict";

exports.value = 1;
exports.assign = function assign() {
	for (exports.value of [2]) {
		// nothing
	}
};
