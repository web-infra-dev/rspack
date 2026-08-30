it("should reexport modules and properties via Object.defineProperty", () => {
	expect(require("./reexport-whole-define?1").module1.abc).toBe("abc");
	expect(require("./reexport-whole-define?2").module2.abc).toBe("abc");
	expect(require("./reexport-whole-define?3").module3.abc).toBe("abc");
	expect(require("./reexport-property-define?1").property1).toBe("abc");
	expect(require("./reexport-property-define?2").property2).toBe("abc");
	expect(require("./reexport-property-define?3").property3).toBe("abc");
});

it("should reexport reexports and deeply nested properties", () => {
	expect(require("./reexport-reexport-define?1").reexport1.abc).toBe("abc");
	expect(require("./reexport-reexport-define?2").reexport2.abc).toBe("abc");
	expect(require("./reexport-reexport-define?3").reexport3.abc).toBe("abc");
	expect(require("./reexport-nested?1").nested).toBe("nested-value");
});

it("should reexport via lazy getters", () => {
	expect(require("./reexport-getter-define?1").getter1.abc).toBe("abc");
	expect(require("./reexport-getter-define?2").getter2).toBe("abc");
	expect(require("./reexport-getter-define?3").getter3.abc).toBe("abc");
});

it("should keep eager effects but not mark their unused export", () => {
	const counter = require("./counter");
	counter.value = 0;
	Object.defineProperty(exports, "unused1", { value: require("./add-to-counter?1") });
	Object.defineProperty(exports, "unused2", { value: require("./add-to-counter?2").abc });
	expect(counter.value).toBe(2);
	if (process.env.NODE_ENV === "production") {
		expect(require("./add-to-counter?1").abcUsed).toBe(false);
		expect(require("./add-to-counter?2").abcUsed).toBe(false);
	}
});

it("should defer getter execution and preserve setters", () => {
	const counter = require("./counter");
	counter.value = 0;
	const lazy = require("./reexport-lazy-getter");
	expect(counter.value).toBe(0);
	expect(lazy.lazy.abc).toBe(42);
	expect(counter.value).toBe(1);
	const m = require("./reexport-getter-setter?1");
	expect(m.value).toBe("abc");
	m.value = "written";
	expect(m.getLastSet()).toBe("written");
});

it("should never execute an unused getter reexport", () => {
	const counter = require("./counter");
	counter.value = 0;
	require("./reexport-unused-getter");
	expect(counter.value).toBe(0);
});
