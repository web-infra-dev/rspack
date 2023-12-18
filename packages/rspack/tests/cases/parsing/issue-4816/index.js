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

	expect(!require("./a") && !require("./b")).toBe(false);

	expect("hello" && (() => "value5")()).toBe("value5");
	expect("" || (() => "value6")()).toBe("value6");
	expect(
		(function () {
			return "value7" === typeof "value7" && "value7";
		})()
	).toBe(false);
	expect([] != [] || require("fail")).toBe(true);
	expect(null === 1 && require("fail")).toBe(false);
	expect([] === [] && require("fail")).toBe(false);
	expect(/a/ === /a/ && require("fail")).toBe(false);
	// NEXT:
	// expect(`hello${Math.random()}` === `world${Math.random()}` && require("fail")).toBe(false);
});
