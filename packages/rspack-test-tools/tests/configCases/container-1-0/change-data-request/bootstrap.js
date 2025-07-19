import a from "myA";

export function test(it) {
	it("should have correct value for remote module", () => {
		expect(a).toBe("a");
	});
}
