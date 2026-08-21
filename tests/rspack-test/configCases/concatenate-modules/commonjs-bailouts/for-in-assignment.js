"use strict";

exports.value = "";
exports.assign = function assign() {
	for (exports.value in { key: true }) {
		// nothing
	}
};
