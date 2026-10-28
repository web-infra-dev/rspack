it("should compile and minimize with the inherited ES2026 target", () => {
	class Counter {
		#value = 42;
		get value() {
			return this.#value;
		}
	}
	const counter = new Counter();
	expect(counter?.value ?? 0).toBe(42);
});
