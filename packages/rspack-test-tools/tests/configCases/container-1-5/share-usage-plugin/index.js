it("should work with Module Federation shared modules and generate share-usage.json", async () => {
	const { map, filter, uniq, debounce } = await import("lodash-es");
	
	const data = [1, 2, 3, 4, 5];
	const doubled = map(data, x => x * 2);
	const filtered = filter(doubled, x => x > 5);
	
	expect(doubled).toEqual([2, 4, 6, 8, 10]);
	expect(filtered).toEqual([6, 8, 10]);
	
	expect(typeof uniq).toBe("function");
	expect(typeof debounce).toBe("function");
	
	const React = await import("react");
	const element = React.createElement("div", null, "Hello World");
	expect(element.type).toBe("div");
	expect(element.props.children).toBe("Hello World");
	
	const utilsModule = await import("./utils.js");
	const validationResult = utilsModule.validateData([1, 2, 3]);
	expect(validationResult.valid).toBe(true);
	expect(validationResult.type).toBe("array");
	expect(validationResult.length).toBe(3);
	
	const componentsModule = await import("./components.js");
	const helloElement = componentsModule.HelloComponent({ name: "Test" });
	expect(helloElement.type).toBe("div");
	expect(helloElement.props.children).toBe("Hello, Test!");
	
	const cjsModule = await import("./cjs-test-module.js");
	const cjsResult = cjsModule.processData([1, 2, 3]);
	expect(cjsResult).toEqual([2, 4, 6]);
	expect(cjsModule.helperFunction()).toBe("helper");
	expect(cjsModule.additionalExport()).toBe("additional");
	
	const esmModule = await import("./esm-test-module.js");
	const cloned = esmModule.useClone({ a: 1 });
	expect(cloned).toEqual({ a: 1 });
	const merged = esmModule.useMerge({ a: 1 }, { b: 2 });
	expect(merged).toEqual({ a: 1, b: 2 });
	expect(esmModule.default()).toBe("ESM default export");
	expect(esmModule.barrelExportA).toBe("barrel A");
	
	const localCjs = await import("./local-cjs-module.js");
	expect(localCjs.usedLocalFunction()).toBe("This local function is used");
	expect(localCjs.constantValue).toBe("test value");
	expect(localCjs.nestedObject.prop1).toBe("value1");
	expect(typeof localCjs.unusedLocalFunction).toBe("function");
	expect(typeof localCjs.unusedConstant).toBe("string");
	
	const localEsm = await import("./local-esm-module.js");
	expect(localEsm.usedLocalUtil()).toBe("This local utility is used");
	expect(localEsm.USED_LOCAL_CONSTANT).toBe("used local constant");
	expect(localEsm.default()).toBe("local default export function");
	expect(typeof localEsm.unusedLocalUtil).toBe("function");
	expect(typeof localEsm.UNUSED_LOCAL_CONSTANT).toBe("string");
	
	const cjsExports = await import("./cjs-exports-test.js");
	expect(cjsExports.default()).toBe(6);
	expect(cjsExports.utilityA()).toBe("utility A");
});