it("should generate correct worker runtime code with tree shaking and MF runtime plugin", async () => {
	const { getMessage, getWorkerMessage } = await import('./bootstrap.js');
	expect(getMessage()).toBe('App rendered with [This is react 0.2.1]');

	const plugins = __webpack_require__.federation.initOptions.plugins;
	const pluginNames = plugins.map(p => p.name);
	const expectedPluginNames = ['my-runtime-plugin', 'my-runtime-plugin-esm', 'tree-shake-plugin'];
	expect(pluginNames).toEqual(expect.arrayContaining(expectedPluginNames));
	expect(pluginNames.filter(name => expectedPluginNames.includes(name))).toEqual(expectedPluginNames);

	expect(await getWorkerMessage()).toBe('Echo: Hello, Rspack!');
});
