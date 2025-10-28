it("context module + dynamic import + binary expression", async function () {
	let params = "index";
	await import("./child/" + params + ".js").then(module => {
		expect(module.value).toBe("dynamic");
	});
});
