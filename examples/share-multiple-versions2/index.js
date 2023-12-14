(async () => {
	await __webpack_init_sharing__("default");
	const { version: versionInner } = await import("my-module");
	console.log(versionInner);
})();
