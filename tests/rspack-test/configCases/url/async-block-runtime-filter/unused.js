import { value } from "./shared";
it("should not need the asset in the other runtime", () => {
	expect(value()).toBe("unused runtime");
});
