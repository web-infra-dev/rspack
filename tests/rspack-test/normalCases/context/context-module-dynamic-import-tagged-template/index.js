it("context module + dynamic import + tagged template", async function () {
	let a = "child/index";
	await import(String.raw`./${a}.js`).then(module => {
		expect(module.value).toBe("dynamic");
	});

	let tagFunc = function () {
		return "./child/index";
	};
	await import(tagFunc`./${a}.js`).catch(err => {
		expect(err.message).toMatch(/Cannot find module/);
	});
});
