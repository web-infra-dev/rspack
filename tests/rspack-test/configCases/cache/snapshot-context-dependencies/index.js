const name = "a";

it("should support snapshot.contextDependencies configuration", async () => {
	const mod = await import(`./dir/${name}.js`);
	expect(mod.default).toBe(1);
});
