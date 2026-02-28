import value from "./value.json"

it("should not shake json", () => {
	expect(value.types.includes('foo')).toBe(true)
})
