it("condition expr should works in require", () => {
	const ok = () => {};
	const res = require(ok() ? "./a" : `./b`);
	expect(res).toBe("b");
});

it("should build success for logic op", () => {
	expect("" && require("fail")).toBe("");
});
