class KeepClass {
	constructor() {
		this.name = "test-keep-class-names";
	}

	getName() {
		return this.name;
	}
}

it("should keep class names", () => {
	const name = KeepClass.name;
	console.log("name", name);
	expect(name).toBe("KeepClass");
});
