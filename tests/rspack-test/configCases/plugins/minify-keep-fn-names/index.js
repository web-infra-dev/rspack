function main() {}

it("should keep fn names", () => {
	const name = main.name;
	expect(name).toBe("main");
});
