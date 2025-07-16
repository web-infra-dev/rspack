it("should work with Module Federation shared modules and generate share-usage.json", async () => {
	// Test lodash-es imports - some used, some unused
	const { map, filter, uniq, debounce } = await import("lodash-es");
	
	// Use some of the imported functions
	const data = [1, 2, 3, 4, 5];
	const doubled = map(data, x => x * 2);
	const filtered = filter(doubled, x => x > 5);
	
	// Verify the used functions work correctly
	expect(doubled).toEqual([2, 4, 6, 8, 10]);
	expect(filtered).toEqual([6, 8, 10]);
	
	// Note: uniq and debounce are imported but not used
	// This should show up in the ShareUsagePlugin output as unused_exports
	expect(typeof uniq).toBe("function");
	expect(typeof debounce).toBe("function");
	
	// Test React import
	const React = await import("react");
	const element = React.createElement("div", null, "Hello World");
	expect(element.type).toBe("div");
	expect(element.props.children).toBe("Hello World");
	
	// Test utils module
	const utilsModule = await import("./utils.js");
	const validationResult = utilsModule.validateData([1, 2, 3]);
	expect(validationResult.valid).toBe(true);
	expect(validationResult.type).toBe("array");
	expect(validationResult.length).toBe(3);
	
	// Test components module
	const componentsModule = await import("./components.js");
	const helloElement = componentsModule.HelloComponent({ name: "Test" });
	expect(helloElement.type).toBe("div");
	expect(helloElement.props.children).toBe("Hello, Test!");
	
	// Note: ShareUsagePlugin validation happens in the afterBuild hook
	// This test verifies that the code works and imports are correctly tracked
});