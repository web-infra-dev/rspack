import * as ns from "./foo.mjs";

function getProto() {
	return "application";
}

function createFakeNamespaceObject() {
	return "application";
}

const moduleCache = "application";
const modules = "application";
const rspackRequire = "application";

it("should avoid conflicts with runtime module variables", () => {
	expect(ns).toBeDefined();
	expect(getProto()).toBe("application");
	expect(createFakeNamespaceObject()).toBe("application");
	expect(moduleCache).toBe("application");
	expect(modules).toBe("application");
	expect(rspackRequire).toBe("application");
});

export {
	createFakeNamespaceObject,
	getProto,
	moduleCache,
	modules,
	rspackRequire,
};
