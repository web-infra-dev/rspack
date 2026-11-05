import { value } from "library";

it("should not await the settled exports of an async module library again", () => {
	expect(value).toBe(42);
});
