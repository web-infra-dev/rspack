import { m } from "./reexport"

it("should have correct value for destructuring assignment a call expr", () => {
	const { value } = m.f();
	expect(value).toBe(42);
})
