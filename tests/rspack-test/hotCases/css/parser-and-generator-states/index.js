import style from './index.module.css';

it("should store and resume css parser and generator states", async () => {
	expect(style['btnInfoIsDisabled']).toBe('./index.module.css__btn-info_is-disabled');
	await NEXT_HMR();
	expect(style['btnInfoIsEnabled']).toBe('./index.module.css__btn-info_is-enabled');
});

module.hot.accept('./index.module.css')
