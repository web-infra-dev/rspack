it("should import by full require with require()()", () => {
	const res = require("./exports.js")();
	expect(res).toBe("abc");
});

it("should import by full require with require().a", () => {
	const res = require("./exports.js").a;
	expect(res).toBe("abc");
});

it("should import by full require with require().b()", () => {
	const res = require("./exports.js").b();
	expect(res).toBe("abc");
});

it("should import by full require with require().c.d()", () => {
	const res = require("./exports.js").c.d();
	expect(res).toBe("abc");
});

it("should import by full require with require().e.f().g.h", () => {
	const res = require("./exports.js").e.f().g.h;
	expect(res).toBe("abc");
});

it("should import by full require with require().h.i().j.k()", () => {
	const res = require("./exports.js").h.i().j.k();
	expect(res).toBe("abc");
});

it("should import by full require with require().l.m()().n", () => {
	const res = require("./exports.js").l.m()().n;
	expect(res).toBe("abc");
});

it("should not throw error when require in try catch", () => {
	try {
		let res = require("fail").a;
	} catch (e) {}
});
