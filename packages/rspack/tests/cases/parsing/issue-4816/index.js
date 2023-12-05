it("condition expr should works in require", () => {
	const ok = () => {};
	const res = require(ok() ? "./a" : `./b`);
	expect(res).toBe("b");
});
