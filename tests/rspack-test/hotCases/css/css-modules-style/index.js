import style from './index.module.css';

module.hot.accept('./index.module.css')

it("css modules hmr", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	expect(style.div).toBeDefined();
	NEXT(require("../../update")(done, true, () => {
		expect(style.a).toBeDefined();
		expect(style).not.toContain('div');
		done();
	}));
}));
