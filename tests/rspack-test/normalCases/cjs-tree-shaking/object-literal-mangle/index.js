it("should rename every kind of used property consistently", async () => {
	expect(require("./lib").keyValue).toBe("key-value");
	expect(require("./lib").shorthand).toBe("shorthand");
	expect(require("./lib")["quoted-key"]).toBe("quoted");
	expect(require("./lib").method()).toBe("method");
	expect(require("./lib").getter).toBe("getter");
	require("./lib").setter = "set";
	expect(require("./lib").readSetter()).toBe("set");
	expect(await require("./lib").asyncMethod()).toBe("async");
	expect([...require("./lib").generatorMethod()]).toEqual(["generator"]);
	expect(require("./lib").toString).toBe("to-string");
	expect(require("./lib").canMangle).toBe(true);
});

it("should read renamed properties through an ESM import", () => {
	expect(require("./esm").keyValue).toBe("key-value");
	expect(require("./esm").shorthand).toBe("shorthand");
	expect(require("./esm").method).toBe("method");
});
