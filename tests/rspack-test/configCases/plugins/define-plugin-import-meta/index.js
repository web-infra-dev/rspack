it("should have import.meta.env", function() {
	expect(import.meta.env).toEqual({
		MODE: "production",
		NODE_ENV: "production"
	});
	expect(import.meta.env.MODE).toBe("production");
	expect(import.meta.env.NODE_ENV).toBe("production");
});
