import style from './index.module.css';

it("css modules hmr", (done) => {
	expect(style.div).toBeDefined();
	NEXT(require("../../update")(done, true, () => {
		const style = require('./index.module.css')
		expect(style.a).toBeDefined();
		expect(style).not.toContain('div');
		done();
	}));
});
