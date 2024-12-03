import style from './index.module.css';

module.hot.accept('./index.module.css')

it("should store and resume css parser and generator states", (done) => {
	expect(style['btnInfoIsDisabled']).toBe('./index.module.css__btn-info_is-disabled');
	NEXT(require("../../update")(done, true, () => {
		expect(style['btnInfoIsEnabled']).toBe('./index.module.css__btn-info_is-enabled');
		done();
	}));
});
