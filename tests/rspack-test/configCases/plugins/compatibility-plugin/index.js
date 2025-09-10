it("compatibility plugin", async () => {
  const f = require("./a.js");

  expect(f(1)).toBe(2);

	const context = require('./c.js');
	const { __webpack_require__ } = context;
	expect(__webpack_require__).toBe(1);

	(function f({ __webpack_require__ }) {
		expect(__webpack_require__).toBe(1);
	})({ __webpack_require__ })
});
