it("should load the full async commons", () => new Promise(done => {
	require.ensure(["./a"], (require) => {
		expect(require("./a")).toBe("a");
		done();
	});
}));

it("should load a chunk with async commons (AMD)", () => new Promise(done => {
	require(["./a", "./b"], (a, b) => {
		expect(a).toBe("a");
		expect(b).toBe("b");
		done();
	});
}));

it("should load a chunk with async commons (require.ensure)", () => new Promise(done => {
	require.ensure([], (require) => {
		expect(require("./a")).toBe("a");
		expect(require("./c")).toBe("c");
		done();
	});
}));
