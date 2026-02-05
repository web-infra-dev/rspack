it("should generate correct worker runtime code with tree shaking and MF runtime plugin", async () => {
	const { getMessage, getWorkerMessage } = await import('./bootstrap.js');
	expect(getMessage()).toBe('App rendered with [This is react 0.2.1]');

	const plugins = __webpack_require__.federation.initOptions.plugins;
  // mf webpack bundler runtime has 1 built-in plugin
	expect(plugins.length).toBe(3);
	expect(plugins.map(p => p.name)).toEqual(['my-runtime-plugin', 'my-runtime-plugin-esm','tree-shake-plugin']);

	expect(await getWorkerMessage()).toBe('Echo: Hello, Rspack!');
});
