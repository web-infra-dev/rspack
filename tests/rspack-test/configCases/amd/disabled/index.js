it("should compile", () => new Promise(done => {
	done();
}));

it("should disable define", () => new Promise(done => {
	expect(typeof define).toBe('undefined')
	done()
}));
