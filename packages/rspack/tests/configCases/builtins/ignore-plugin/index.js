it("ignore plugin should works", () => {
	const a = require('./test-ignore/a');

	expect(() => a.requireB()).toThrow(`Cannot find module './b'`)
	expect(a.requireC().default).toBe('c');
	expect(() => a.requireD()).toThrow(`Cannot find module './d'`)
});

