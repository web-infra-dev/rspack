it("should export a container object with async startup", () => {
	const container = __non_webpack_require__("./remoteEntry.js");
	expect(container).toBeTruthy();
	expect(typeof container.then).not.toBe("function");
	expect(typeof container.get).toBe("function");
	expect(typeof container.init).toBe("function");
});
