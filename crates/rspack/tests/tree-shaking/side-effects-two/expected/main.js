(self["webpackChunkwebpack"] = self["webpackChunkwebpack"] || []).push([
	["main"],
	{
		"./app.js": function (module, exports, __webpack_require__) {
			"use strict";
			Object.defineProperty(exports, "__esModule", {
				value: true
			});
			Object.defineProperty(exports, "something", {
				enumerable: true,
				get: () => _lib.default
			});
			const _lib = __webpack_require__.interopRequire(
				__webpack_require__("./lib.js")
			);
		},
		"./index.js": function (module, exports, __webpack_require__) {
			"use strict";
			Object.defineProperty(exports, "__esModule", {
				value: true
			});
			const _app = __webpack_require__("./app.js");
			(0, _app.something)();
		},
		"./lib.js": function (module, exports, __webpack_require__) {
			"use strict";
			Object.defineProperty(exports, "__esModule", {
				value: true
			});
			Object.defineProperty(exports, "default", {
				enumerable: true,
				get: () => _default
			});
			function _default() {}
		}
	},
	function (__webpack_require__) {
		__webpack_require__("./index.js");
	}
]);
