function test() {
	return 123;
}

it("basic", () => {
	expect(test()).toBe(123);
});
