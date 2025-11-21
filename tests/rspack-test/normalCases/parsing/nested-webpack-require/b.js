const map = {
	key: function (__webpack_require__) {
		function d() {
			function __webpack_require__() {}
			it("webpack also report undefined, this maybe a bug", () => {
				expect(require("./a.js")).toBeUndefined();
			});
		}
		function e(__webpack_require__) {}
		d();
	}
};

it("nested nested should works", function () {
	const { key } = map;
	expect(typeof key).toBe("function");
	map.key();
});
