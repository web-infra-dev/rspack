it("skip windows", async () => {
	if (process.platform !== "win32") {
		await import("./entry");
	}
	expect("ok").toBe("ok");
});
