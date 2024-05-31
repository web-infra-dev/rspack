import { a, b } from "./a.js";

it('should load a.js', () => {
	expect(a).toBe(42)

	expect(b).toBe("foo")
})
