const name = "a";

it("should support snapshot.contextModule configuration", async () => {
	const mod = await import(`./dir/${name}.js`);
	expect(mod.default).toBe(1);
});
