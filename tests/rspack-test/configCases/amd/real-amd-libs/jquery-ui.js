/*!
 * Code simplified from jQuery UI 1.14.1
 * for testing amd support
 */
(function (factory) {
	"use strict";
	if (typeof define === "function" && define.amd) {
		// AMD. Register as an anonymous module.
		define(["./jquery"], factory);
	} else {
		// Browser globals
		factory(jQuery);
	}
})(function ($) {
	return { version: '0.0.0' };
});
