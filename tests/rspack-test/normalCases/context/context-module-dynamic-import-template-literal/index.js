it("context module + dynamic import + template literal", async function () {
	let params = "index";
	await import(`./child/${params}.js`).then(module => {
		expect(module.value).toBe("dynamic");
	});
});
