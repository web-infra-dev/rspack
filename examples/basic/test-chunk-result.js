(self["webpackChunktest"] = self["webpackChunktest"] || []).push([
	["chunk"],
	{
		module1: function (module, exports, __webpack_require__) {
			console.log("Module 1");
			__webpack_require__("module2");
		},
		module2: function (module, exports, __webpack_require__) {
			console.log("Module 2");
		},
		module3: function (module, exports, __webpack_require__) {
			console.log("Module 3 - no dependencies on it");
		}
	}
]);
