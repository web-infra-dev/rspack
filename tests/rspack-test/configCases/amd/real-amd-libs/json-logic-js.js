/*!
 * Code simplified from https://github.com/jwadhams/json-logic-js/blob/master/logic.js
 * for testing amd support
 */
; (function (root, factory) {
	if (typeof define === "function" && define.amd) {
		define(factory);
	} else if (typeof exports === "object") {
		module.exports = factory();
	} else {
		root.jsonLogic = factory();
	}
}(this, function () {
	var jsonLogic = { version: '0.0.0' };
	return jsonLogic;
}));
