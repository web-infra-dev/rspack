it("should load an async stylesheet that is initial for another entry", async () => {
	expect(document.getElementsByTagName("link")).toHaveLength(0);

	const promise = import("./feature.js");
	const links = document.getElementsByTagName("link");
	expect(links).toHaveLength(1);
	expect(links[0].rel).toBe("stylesheet");
	expect(links[0].href).toContain("shared-css.");

	expect((await promise).value).toBe(42);
	expect(getComputedStyle(document.body).getPropertyValue("color")).toBe("rgb(255, 0, 0)");

	await import("./feature.js");
	expect(document.getElementsByTagName("link")).toHaveLength(1);
});
