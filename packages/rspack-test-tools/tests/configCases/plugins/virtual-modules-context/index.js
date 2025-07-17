
it('virtual context modules should work', () => {
	for (const [lang, hello] of Object.entries({
		en: 'hello',
		zh: '你好',
	})) {
		expect(require(`./translations/${lang}.js`).hello).toBe(hello);
	}
});
