import * as namespace from "./barrel.js";
import * as localNamespace from "./local-barrel.js";

it("should preserve reexports read through the namespace call context", () => {
	expect(namespace.readOther()).toBe(42);
});

it("should build the imports of a barrel read through the namespace call context", () => {
	expect(localNamespace.readLocal()).toBe("helper");
});
