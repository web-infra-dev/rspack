import style from './index.module.css';

it("should store and resume css parser and generator states", (done) => {
	expect(style['btnInfoIsDisabled']).toBe('./index.module.css__btn-info_is-disabled');
	module.hot.accept("./index.module.css", () => {
		expect(style['btnInfoIsEnabled']).toBe('./index.module.css__btn-info_is-enabled');
		done();
	});
	NEXT(require("../../update")(done));
});
