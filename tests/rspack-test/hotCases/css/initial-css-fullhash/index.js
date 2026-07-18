import './index.css';

it("includes the full hash runtime for initial CSS filenames", () => {
	if (__webpack_require__.hmrC.css) {
		expect(typeof __webpack_require__.h).toBe("function");
	}
});

module.hot.accept("./index.css");
