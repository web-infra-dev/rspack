import { a, b } from "./reexport";

if (process.env.NODE_ENV !== "production") {
	a;
}

it("should works", () => {
	expect(b).toBe(43);
});
