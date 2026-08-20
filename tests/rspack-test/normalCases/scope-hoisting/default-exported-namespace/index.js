import queryString from "./query-string";

it("should keep an operand for a pure default-exported namespace", () => {
	expect(queryString.parse("value")).toBe("value");
});
