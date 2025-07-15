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
	expect(
		`hello${Math.random()}` === `world${Math.random()}` && require("fail")
	).toBe(false);
	expect(
		`${Math.random()}hello` != `${Math.random()}world` || require("fail")
	).toBe(true);
	let value95 = 1;
	expect(`${value95++}hello` != `${value95++}world` || require("fail")).toBe(
		true
	);
	if (`${value95++}hello` === `${value95++}world`) {
		require("fail");
	}
	expect(value95).toBe(5);
});

it("should keep the variable in dead branch", () => {
	if (/b/ === /b/) {
		function f1() {}
		const f3 = () => {};
		const g = function e() {};
		var obj = { y: 3, z: 4 };
		if (true) {
			let a = 1;
			var x = 2;
			var { y, z } = obj;
		}
	}

	if (/a/ === /a/) {
		("use strict");
		function f2() {}
	}

	function f4() {
		"use strict";
		if (/a/ === /a/) {
			function f5() {}
		}
		expect(() => f5).toThrowError();
	}

	f4();
	expect(f1).toBeUndefined();
	expect(f2).toBeUndefined();
	expect(() => f3).toThrowError();
	expect(x).toBeUndefined();
	expect(y).toBeUndefined();
	expect(z).toBeUndefined();
	expect(obj).toBeUndefined();
	expect(() => e0).toThrowError();
	expect(() => e).toThrowError();
	expect(() => e1).toThrowError();
	expect(() => e2).toThrowError();
	expect(() => g).toThrowError();
	expect(() => a).toThrowError();
	expect(() => a1).toThrowError();
	expect(() => a2).toThrowError();
	expect(() => a3).toThrowError();
});

it("shouldn't evaluate expression", function () {
	const value = "";
	const x = value + "" ? "fail" : "ok";
	expect(x).toBe("ok");
});

it("should short-circuit evaluating", function () {
	let expr;
	const a = false && expr ? require("fail") : require("./a");
	const b = true || expr ? require("./a") : require("fail");
	expect(a).toBe("a");
	expect(b).toBe("a");
});

it("should not evaluate new RegExp for redefined RegExp", () => {
	const RegExp = function () {
		return /other/;
	};
	expect(require("./regexp/" + "a".replace(new RegExp("a"), "wrong"))).toBe(1);
});

it("should try to evaluate new RegExp()", function () {
	function expectAOnly(r) {
		r.keys().forEach(key => {
			expect(key).toBe("./a.js");
			expect(r(key)).toBe(1);
		});
	}

	expectAOnly(
		require.context("./regexp", false, new RegExp("(?<!filtered)\\.js$", ""))
	);
	expectAOnly(
		require.context(
			"./regexp",
			false,
			new RegExp(`(?<!${"FILTERED"})\\.js$`, "i")
		)
	);
	expectAOnly(
		require.context("./regexp", false, new RegExp("(?<!filtered)\\.js$"))
	);
	expectAOnly(
		require.context(`./regexp`, false, new RegExp("(?<!filtered)\\.js$"))
	);
});

it("should evaluate __dirname and __resourceQuery with replace and substr", function () {
	const result = require("./resourceQuery/index?" + __dirname);
	expect(result).toEqual("?resourceQuery");
});

it("should evaluate __dirname and __resourceFragment with replace and substr", function () {
	const result = require("./resourceFragment/index#" + __dirname);
	expect(result).toEqual("#resourceFragment");
});

it("should parse nullish coalescing correctly", () => {
	let result;

	if ((null ?? false) === null) {
		result = require("./b");
	} else if ((0 ?? false) === 0) {
		result = require("./a");
	}

	expect(result).toBe("a");
});

// function a() { }

it("should evaluate nullish coalescing", function () {
	expect("" ?? require("fail")).toBe("");
	// expect(String.raw`aaaa` ?? require("fail")).toBe("aaaa");
	// expect(a`aaaa` ?? "expected").toBe("expected");
	expect(null ?? "expected").toBe("expected");
	expect(("" ?? require("fail")) && true).toBe("");
	let x = 0;
	expect(((x = 1), null) ?? true).toBe(true);
	expect(x).toBe(1);
});
