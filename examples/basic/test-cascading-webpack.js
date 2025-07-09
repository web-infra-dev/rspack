// More realistic webpack bundle format
(function () {
	var __webpack_modules__ = {
		// Entry point - always reachable
		100: function (module, exports, __webpack_require__) {
			/* @common:if [condition="treeShake.test-lib.featureA"] */
			var featureA = __webpack_require__("200");
			console.log("Using feature A");
			/* @common:endif */
		},

		// Feature A module - conditionally used
		200: function (module, exports, __webpack_require__) {
			var helperB = __webpack_require__("300");
			var helperC = __webpack_require__("400");
			module.exports = { helper: helperB, util: helperC };
		},

		// Helper B - depends on Feature A
		300: function (module, exports, __webpack_require__) {
			var utilD = __webpack_require__("500");
			module.exports = function () {
				return utilD() + "B";
			};
		},

		// Helper C - depends on Feature A
		400: function (module, exports, __webpack_require__) {
			module.exports = { name: "HelperC" };
		},

		// Util D - deep dependency
		500: function (module, exports, __webpack_require__) {
			module.exports = function () {
				return "utilD";
			};
		},

		// Orphaned module - should be removed immediately
		600: function (module, exports, __webpack_require__) {
			module.exports = { orphaned: true };
		}
	};

	// Entry point call
	__webpack_require__("100");
})();
