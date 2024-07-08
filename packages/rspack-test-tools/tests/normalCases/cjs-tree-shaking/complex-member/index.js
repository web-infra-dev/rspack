it("should read assign exports", () => {
	var e = require("./assign-exports");
	expect(e.abc.a).toBe("a");
	expect(e.abc.b).toBe("b");
});

it("should read assign module.exports", () => {
	var e = require("./assign-module-exports");
	expect(e.abc.a).toBe("a");
	expect(e.abc.b).toBe("b");
});

it("should read assign this", () => {
	var e = require("./assign-this");
	expect(e.abc.a).toBe("a");
	expect(e.abc.b).toBe("b");
});

// it("should read self exports", () => {
//   var e = require("./self-exports");
//   expect(e.abc.a).toBe("a");
//   expect(e.abc.b).toBe("b");
// });

// it("should read self module.exports", () => {
//   var e = require("./self-module-exports");
//   expect(e.abc.a).toBe("a");
//   expect(e.abc.b).toBe("b");
// });

it("should read self this", () => {
	var e = require("./self-this");
	expect(e.abc.a).toBe("a");
	expect(e.abc.b).toBe("b");
});
