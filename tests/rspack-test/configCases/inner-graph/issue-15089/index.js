import { Used } from "./views";

it("should keep references in unused impure class expressions", () => {
	expect(Used.getValue()).toBe("value");
});
