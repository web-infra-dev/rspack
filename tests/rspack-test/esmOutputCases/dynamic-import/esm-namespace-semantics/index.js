it("returns an ESM namespace without the CommonJS interop marker", async () => {
	const namespace = await import("./module.js")

	expect(namespace.default).toBe(42)
	expect(namespace.__esModule).toBeUndefined()
	expect(Object.getPrototypeOf(namespace)).toBeNull()
	expect(Object.isExtensible(namespace)).toBe(false)
	expect(Object.prototype.toString.call(namespace)).toBe("[object Module]")
	expect(namespace.value).toBe(1)
	namespace.increment()
	expect(namespace.value).toBe(2)
	expect(namespace.requireSeesEsModule()).toBe(true)
})
