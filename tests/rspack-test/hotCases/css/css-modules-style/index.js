import style from './index.module.css';

it("css modules hmr", async () => {
	expect(style.div).toBeDefined();
	await NEXT_HMR();
	expect(style.a).toBeDefined();
	expect(style).not.toContain('div');
});

module.hot.accept('./index.module.css')
