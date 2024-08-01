import style from './index.module.css';

it("css modules hmr", (done) => {
	expect(style.div).not.toBe(null);
	module.hot.accept("./index.module.css", () => {
		expect(style.a).not.toBe(null);
		expect(style).not.toContain('div');
		done();
	});
	NEXT(require("../../update")(done));
});
