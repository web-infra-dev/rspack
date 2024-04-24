import './index.css';

it("css hmr", (done) => {
	if (__webpack_require__.hmrC.css) {
		expect(document.head.children[0].href).toContain("bundle.css");
	}
	NEXT(require("../../update")(done, true, () => {
		if (__webpack_require__.hmrC.css) {
			expect(document.head.children[0].href).toContain("bundle.css?hmr");
			expect(document.head.children[0].getAttribute('data-webpack')).toBe("css-test:chunk-main");
		}
		done();
	}));
});
