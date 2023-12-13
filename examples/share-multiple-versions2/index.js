(async () => {
	await __webpack_init_sharing__("default");
	const m = await import("my-module");
	console.log(m);
})();
