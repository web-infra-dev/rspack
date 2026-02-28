it("should load the full async commons", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	require.ensure(["./a"], function(require) {
		expect(require("./a")).toBe("a");
		done();
	});
}));

it("should load a chunk with async commons (AMD)", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	require(["./a", "./b"], function(a, b) {
		expect(a).toBe("a");
		expect(b).toBe("b");
		done();
	});
}));

it("should load a chunk with async commons (require.ensure)", () => new Promise((resolve, reject) => {
	const done = err => (err ? reject(err) : resolve());
	require.ensure([], function(require) {
		expect(require("./a")).toBe("a");
		expect(require("./c")).toBe("c");
		done();
	});
}));
