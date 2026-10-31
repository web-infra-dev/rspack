import value from "./barrel.js";

export default value();

it("should keep unused reexports lazy after restoring a barrel with a forwarded default", () => {
	expect(value()).toBe("value");
});
