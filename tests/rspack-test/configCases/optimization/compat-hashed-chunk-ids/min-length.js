const chunks = [
	import("./a?23"),
	import("./a?30"),
];

it("should extend compat-hashed chunk ids only when shorter prefixes collide", async () => {
	const values = await Promise.all(chunks);
	expect(values.every(value => value.default === "a")).toBe(true);
});
