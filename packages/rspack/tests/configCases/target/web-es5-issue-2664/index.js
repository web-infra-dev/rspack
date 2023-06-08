it("should be called successfully", () => {
	class A {}
	const a = new A();
	expect(a instanceof A).toBeTruthy();
});
