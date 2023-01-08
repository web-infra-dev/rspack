(self["webpackChunkwebpack"] = self["webpackChunkwebpack"] || []).push([
	["main"],
	{
		"./answer.js": function (module, exports, __webpack_require__) {
			"use strict";
			Object.defineProperty(exports, "__esModule", {
				value: true
			});
			Object.defineProperty(exports, "answer", {
				enumerable: true,
				get: () => answer
			});
			const answer = 42;
		},
		"./index.js": function (module, exports, __webpack_require__) {
			"use strict";
			__webpack_require__("./answer.js");
			myanswer();
		}
	},
	function (__webpack_require__) {
		__webpack_require__("./index.js");
	}
]);
