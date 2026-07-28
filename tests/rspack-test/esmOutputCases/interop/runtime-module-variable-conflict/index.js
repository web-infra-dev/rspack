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
const __rspack_get_mini_css_chunk_filename = "application";

it("should avoid conflicts with runtime module variables", () => {
	expect(ns).toBeDefined();
	expect(getProto()).toBe("application");
	expect(createFakeNamespaceObject()).toBe("application");
	expect(moduleCache).toBe("application");
	expect(modules).toBe("application");
	expect(rspackRequire).toBe("application");
	expect(__rspack_get_mini_css_chunk_filename).toBe("application");
});

export {
	__rspack_get_mini_css_chunk_filename,
	createFakeNamespaceObject,
	getProto,
	moduleCache,
	modules,
	rspackRequire,
};
