import style from './index.module.css';

it("css modules hmr", (done) => {
	expect(style.div).not.toBe(null);
	if (__webpack_require__.hmrC.css) {
		expect(document.head.children[0].href).toContain("bundle.css");
	}
	module.hot.accept("./index.module.css", () => {
		expect(style.a).not.toBe(null);
		if (__webpack_require__.hmrC.css) {
			expect(document.head.children[0].href).toContain("bundle.css?hmr");
		}
		done();
	});
	NEXT(require("../../update")(done));
});
