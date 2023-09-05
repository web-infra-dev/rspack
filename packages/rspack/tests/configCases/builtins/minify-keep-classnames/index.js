class KeepClass {}

it("should keep class names", () => {
	const name = KeepClass.name;
	expect(name).toBe("KeepClass");
});
