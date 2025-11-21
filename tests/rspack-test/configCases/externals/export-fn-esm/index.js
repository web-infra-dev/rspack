import { foo } from "./foo.mjs"

it("Should work with export a function when using CreateFakeNamespaceObjectRuntimeModule", async function() {
	expect(foo).toBe("foo")
	const myModule = await import("module");
	// namespace object
	expect(typeof myModule).toBe("object");
	expect(myModule.builtinModules).toBeDefined();
});

it("Should work with export a object when using CreateFakeNamespaceObjectRuntimeModule", async function() {
	const myFs = await import("fs");
	expect(typeof myFs).toBe("object");
	expect(myFs.readFileSync).toBeDefined();
});
