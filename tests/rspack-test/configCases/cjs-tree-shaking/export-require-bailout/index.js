import reexport from "./reexport";

it("should import by export require", () => {
	expect(reexport()).toBe(1);
});
