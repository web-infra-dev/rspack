/*!
 * Code simplified from https://code.jquery.com/jquery-3.7.1.js
 * for testing amd support
 */
(function (global, factory) {
	"use strict";
	if (typeof module === "object" && typeof module.exports === "object") {
		// For CommonJS and CommonJS-like environments where a proper `window`
		// is present, execute the factory and get jQuery.
		// For environments that do not have a `window` with a `document`
		// (such as Node.js), expose a factory as module.exports.
		// This accentuates the need for the creation of a real `window`.
		// e.g. var jQuery = require("jquery")(window);
		// See ticket trac-14549 for more info.
		// module.exports = global.document ?
		// 	factory(global, true) :
		// 	function (w) {
		// 		if (!w.document) {
		// 			throw new Error("jQuery requires a window with a document");
		// 		}
		// 		return factory(w);
		// 	};
		module.exports = factory(global, true);
	} else {
		factory(global);
	}

	// Pass this if window is not defined yet
})(typeof window !== "undefined" ? window : this, function (window, noGlobal) {
	var jQuery = function () { return 'hi jQuery' };
	jQuery.version = '3.7.1';
	window.jQuery = jQuery;
	return jQuery;
});
