import { a, b } from "./lib";

if (process.env.NODE_ENV !== "production") {
	a;
}

it("should works", () => {
	expect(b).toBe(43);
});
