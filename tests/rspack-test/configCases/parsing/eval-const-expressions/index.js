it("should evaluate basic constant expressions like webpack", () => {
	expect(String(42)).toBe("42");
	expect(Number("5") * 2).toBe(10);
	expect(Boolean(0)).toBe(false);
	expect("abcdef".slice(1, 3)).toBe("bc");
	expect("abcdef".substr(2, 3)).toBe("cde");
	expect("abcdef".substring(1, 4)).toBe("bcd");
});

it("should evaluate bigint and arithmetic operators", () => {
	// BigInt addition
	expect(1n + 2n).toBe(3n);

	// Numeric Mod
	expect(5 % 2).toBe(1);
	expect(10 % 4).toBe(2);

	// Unsigned right shift
	expect(-1 >>> 0).toBe(0xffffffff);
	expect(1 >>> 0).toBe(1);
});

it("should evaluate typeof and keep side effects for wrapped expressions", () => {
	let called = 0;
	function sideEffect() {
		called++;
		return "x";
	}

	const t1 = typeof function () {};
	const t2 = typeof +1;
	const t3 = typeof ("a" + sideEffect());

	expect(t1).toBe("function");
	expect(t2).toBe("number");
	expect(t3).toBe("string");
	expect(called).toBe(1);
});

