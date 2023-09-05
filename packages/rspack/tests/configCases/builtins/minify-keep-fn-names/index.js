function main() {
	return "test-keep-fn-names";
}

it("should keep fn names", () => {
	const name = main.name;
	expect(name).toBe("main");
});
