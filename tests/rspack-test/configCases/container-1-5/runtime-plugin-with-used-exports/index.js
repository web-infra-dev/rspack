it("should generate correct worker runtime code with tree shaking and MF runtime plugin", async () => {
	const { getMessage, getWorkerMessage } = await import('./bootstrap.js');
	expect(getMessage()).toBe('App rendered with [This is react 0.2.1]');

	const plugins = __webpack_require__.federation.initOptions.plugins;
	expect(plugins.length).toBeGreaterThan(0);
	expect(plugins.some(p => p.name === 'my-runtime-plugin')).toBe(true);

	expect(await getWorkerMessage()).toBe('Echo: Hello, Rspack!');
});
