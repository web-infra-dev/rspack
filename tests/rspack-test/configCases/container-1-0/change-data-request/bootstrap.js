import a from "myA";
import x from "myX";

export function test(it) {
	it("should have correct value for remote module", () => {
		expect(a).toBe("a");
  });
	it("should have correct value for shared module", () => {
    expect(x).toBe("x");
  });
}
