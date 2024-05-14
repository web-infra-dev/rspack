it("should detect query strings in dynamic import as a static value 1 ", function () {
	return Promise.all([
		import("./a").then(({ default: a }) => {
			expect(a()).toBe("a");
		}),
		import("./abc").then(({ default: a }) => {
			expect(a()).toBe("abc");
		}),
		import("./a?queryString").then(({ default: a }) => {
			expect(a()).toBe("a?queryString");
		})
	]);
});

it("should detect query strings in dynamic import as a static value 2", function () {
	var testFileName = "a";

	return Promise.all([
		import(`./${testFileName}`).then(({ default: a }) => {
			expect(a()).toBe("a");
		}),
		import(`./${testFileName}bc`).then(({ default: a }) => {
			expect(a()).toBe("abc");
		}),
		import(`./${testFileName}?queryString`).then(({ default: a }) => {
			expect(a()).toBe("a?queryString");
		})
	]);
});

it("should detect query strings in dynamic import as a static value 3", function () {
	var testFileName = "a";

	return Promise.all([
		import("./" + testFileName).then(({ default: a }) => {
			expect(a()).toBe("a");
		}),
		import("./" + testFileName + "").then(({ default: a }) => {
			expect(a()).toBe("a");
		}),
		import("./" + testFileName + "bc").then(({ default: a }) => {
			expect(a()).toBe("abc");
		}),
		import("./" + testFileName + "?queryString").then(({ default: a }) => {
			expect(a()).toBe("a?queryString");
		})
	]);
});
