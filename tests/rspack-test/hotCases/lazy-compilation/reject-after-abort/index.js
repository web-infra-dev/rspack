import "./unaccepted";

it("should reject lazy imports once the page can no longer apply hot updates", async () => {
	const pending = import("./feature");
	await new Promise((resolve) => setTimeout(resolve, 1000));
	await expect(NEXT_HMR()).rejects.toThrow("not accepted");
	expect(module.hot.status()).toBe("abort");
	await expect(pending).rejects.toThrow("Reload the page");
	await expect(import("./feature")).rejects.toThrow("Reload the page");
});
