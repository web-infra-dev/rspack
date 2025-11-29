import './index.css';

it("css hmr", async () => {
	if (__webpack_require__.hmrC.css) {
		expect(document.head.children[0].href).toContain("bundle.css");
	}
	await NEXT_HMR();
	if (__webpack_require__.hmrC.css) {
		expect(document.head.children[0].href).toContain("bundle.css?hmr");
		expect(document.head.children[0].getAttribute('data-rspack')).toBe("css-test:chunk-main");
	}
});

module.hot.accept("./index.css");