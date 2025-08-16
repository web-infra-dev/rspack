(self.webpackChunk = self.webpackChunk || []).push([
	["test"],
	{
		mod1: function (module, exports, __webpack_require__) {
			__webpack_require__("mod2");
		},
		mod2: function (module, exports) {
			exports.foo = "bar";
		},
		mod3: function (module, exports) {
			exports.unused = "should be removed";
		}
	}
]);
