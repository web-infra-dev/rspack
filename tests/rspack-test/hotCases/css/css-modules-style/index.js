import style from './index.module.css';

module.hot.accept('./index.module.css')

it("css modules hmr", (done) => {
	expect(style.div).toBeDefined();
	NEXT(require("../../update")(done, true, () => {
		expect(style.a).toBeDefined();
		expect(style).not.toContain('div');
		done();
	}));
});
