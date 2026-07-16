function loadEnv() {
	return { MODE: "define-plugin" };
}

it("should preserve DefinePlugin behavior when experiments.env is disabled", () => {
	expect(import.meta.env.MODE).toBe("define-plugin");
});
