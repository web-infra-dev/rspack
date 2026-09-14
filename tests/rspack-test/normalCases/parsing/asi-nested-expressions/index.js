import { read } from "./read"

// Missing semicolons and redundant parentheses are intentional: imported calls
// can be rendered with a leading parenthesis.
it("preserves ASI in default values parsed through arrow cover grammar", async () => {
	const select = async ({ value = (() => {
		const previous = 0
		read(previous)
		return 1
	})() } = {}) => {
		const marker = {}
		read(value)
		return ((value))
	}
	expect(await select()).toBe(1)
})

it("preserves ASI around yields and parenthesized updates", () => {
	let value = 0
	function* values() {
		yield
		read(value)
		yield ((++value))
		read(value)
		yield ((read(value + 1)))
	}

	expect(Array.from(values())).toEqual([undefined, 1, 2])
})

it("preserves ASI inside parenthesized class field initializers", () => {
	class Value {
		field = ((() => {
			const previous = 0
			read(previous)
			return 3
		}))()
		next = read(this.field)
	}

	expect(new Value().next).toBe(3)
})
