it("should work with Module Federation shared modules and generate share-usage.json", async () => {
	// First, use the modules to establish usage patterns
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

	// Now validate the share-usage.json file
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");

	const shareUsagePath = path.join(__dirname, "share-usage.json");
	const shareUsageContent = fs.readFileSync(shareUsagePath, "utf-8");
	const shareUsage = JSON.parse(shareUsageContent);

	// Validate the structure
	expect(shareUsage).toHaveProperty("treeShake");
	const treeShake = shareUsage.treeShake;

	// Validate lodash-es exports
	expect(treeShake).toHaveProperty("lodash-es");
	const lodashExports = treeShake["lodash-es"];
	expect(lodashExports.map).toBe(true); // used
	expect(lodashExports.filter).toBe(true); // used
	expect(lodashExports.uniq).toBe(true); // used
	expect(lodashExports.debounce).toBe(true); // used
	// Some exports might be false if not used
	if (lodashExports.hasOwnProperty("groupBy")) {
		expect(lodashExports.groupBy).toBe(false); // not imported
	}

	// Validate React exports
	expect(treeShake).toHaveProperty("react");
	const reactExports = treeShake["react"];
	expect(reactExports.default).toBe(true); // default import is used

	// Validate local-esm-module exports - ALL should be true since they're all accessed
	expect(treeShake).toHaveProperty("local-esm-module");
	const localEsmExports = treeShake["local-esm-module"];
	expect(localEsmExports.usedLocalUtil).toBe(true); // called
	expect(localEsmExports.USED_LOCAL_CONSTANT).toBe(true); // accessed
	expect(localEsmExports.default).toBe(true); // called
	expect(localEsmExports.unusedLocalUtil).toBe(true); // accessed (typeof check)
	expect(localEsmExports.UNUSED_LOCAL_CONSTANT).toBe(true); // accessed (typeof check)
	expect(localEsmExports.utilityHelpers).toBe(false); // not accessed

	// Validate local-cjs-module exports
	expect(treeShake).toHaveProperty("local-cjs-module");
	const localCjsExports = treeShake["local-cjs-module"];
	// CommonJS modules have limited tracking - check what's available
	if (localCjsExports.hasOwnProperty("usedLocalFunction")) {
		expect(localCjsExports.usedLocalFunction).toBe(true); // called
	}
	if (localCjsExports.hasOwnProperty("constantValue")) {
		expect(localCjsExports.constantValue).toBe(true); // accessed
	}
	if (localCjsExports.hasOwnProperty("nestedObject")) {
		expect(localCjsExports.nestedObject).toBe(true); // accessed
	}
	if (localCjsExports.hasOwnProperty("directProperty")) {
		expect(localCjsExports.directProperty).toBe(false); // not accessed
	}
});
