it("should allow to read own exports via exports", () => {
	var e = require("./reading-self-from-exports");
	expect(e.test()).toBe("abc");
	expect(e.whole().abc).toBe("abc");
	expect(e.fn()).toBe("abc");
});

it("should allow to read own exports via module.exports", () => {
	var e = require("./reading-self-from-module-exports");
	expect(e.test()).toBe("abc");
	expect(e.whole().abc).toBe("abc");
	expect(e.fn()).toBe("abc");
});

it("should allow to read own exports via this", () => {
	var e = require("./reading-self-from-this");
	expect(e.test()).toBe("abc");
	expect(e.whole().abc).toBe("abc");
	expect(e.fn()).toBe("abc");
});

it("should allow to read own exports via exports()", () => {
	var e = require("./reading-self-from-exports-call");
	expect(e.test()).toBe("abc");
});

it("should allow to read own exports via module.exports()", () => {
	var e = require("./reading-self-from-module-exports-call");
	expect(e.test()).toBe("abc");
});

// this can not be assigned
// it("should allow to read own exports via this()", () => {
//   var e = require("./reading-self-from-this-call");
//   expect(e.test()).toBe("abc");
// });
