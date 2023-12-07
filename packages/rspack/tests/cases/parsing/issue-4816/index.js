it("condition expr should works in require", () => {
	const ok = () => {};
	const res = require(ok() ? "./a" : `./b`);
	expect(res).toBe("b");
});

it("should evaluate null", function () {
	expect(null ? require("fail") : require("./a")).toBe("a");
	if (null) require("fail");
});

it("should evaluate undefined", function () {
	expect(undefined ? require("fail") : require("./a")).toBe("a");
	if (undefined) require("fail");
	undefined && require("fail");
});

it("should build success for logic op", () => {
	expect("hello" || require("fail")).toBe("hello");

	expect(typeof require === "function" || require("fail")).toBe(true);
	expect(false || require("./a")).toBe("a");
	expect(typeof require !== "function" || require("./a")).toBe("a");

	expect("" && require("fail")).toBe("");

	expect(typeof require !== "function" && require("fail")).toBe(false);
	expect(true && require("./a")).toBe("a");
	expect(typeof require === "function" && require("./a")).toBe("a");
});
