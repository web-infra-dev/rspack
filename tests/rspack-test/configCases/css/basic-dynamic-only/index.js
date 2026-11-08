it("should compile and load style on demand", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	import("./style.css").then(x => {
		expect(x).toEqual(nsObj({}));
		const style = getComputedStyle(document.body);
		expect(style.getPropertyValue("background-color")).toBe("rgb(255, 0, 0)");
		expect(style.getPropertyValue("margin")).toBe("10px");
		done();
	}, done);
}));
