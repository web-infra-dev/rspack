it("should keep loading the shared chunk after rebuild", async () => {
	const { default: value } = await import(
		/* webpackChunkName: "shared" */ "./shared"
	);

	expect(value).toBe("shared");
});

export const version = "step-1";
