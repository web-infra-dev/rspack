(() => {
	var __webpack_modules__ = {
		100: function (
			__unused_webpack_module,
			__webpack_exports__,
			__webpack_require__
		) {
			/* @common:if [condition="treeShake.test-lib.featureA"] */
			var featureA = __webpack_require__(200);
			console.log("Using feature A");
			/* @common:endif */
		},

		200: function (
			__unused_webpack_module,
			__webpack_exports__,
			__webpack_require__
		) {
			var helperB = __webpack_require__(300);
			var helperC = __webpack_require__(400);
			module.exports = { helper: helperB, util: helperC };
		},

		300: function (
			__unused_webpack_module,
			__webpack_exports__,
			__webpack_require__
		) {
			var utilD = __webpack_require__(500);
			module.exports = function () {
				return utilD() + "B";
			};
		},

		400: function (
			__unused_webpack_module,
			__webpack_exports__,
			__webpack_require__
		) {
			module.exports = { name: "HelperC" };
		},

		500: function (
			__unused_webpack_module,
			__webpack_exports__,
			__webpack_require__
		) {
			module.exports = function () {
				return "utilD";
			};
		},

		600: function (
			__unused_webpack_module,
			__webpack_exports__,
			__webpack_require__
		) {
			module.exports = { orphaned: true };
		}
	};

	// Entry point call
	__webpack_require__(100);
})();
