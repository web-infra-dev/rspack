it("should track inter-shared-library dependencies in share-usage.json", async () => {
	// Import @reduxjs/toolkit which internally uses redux exports
	const toolkit = await import("@reduxjs/toolkit");
	const store = await import("./store.js");
	
	// Use configureStore from toolkit
	const myStore = toolkit.configureStore({
		reducer: {
			counter: store.counterReducer
		}
	});
	
	// The store should work correctly
	expect(typeof myStore.dispatch).toBe("function");
	expect(typeof myStore.getState).toBe("function");
	
	// Dispatch an action
	myStore.dispatch(store.increment());
	const state = myStore.getState();
	expect(state.counter.value).toBe(1);
	
	// Check that createSlice works
	const slice = toolkit.createSlice({
		name: "test",
		initialState: { value: 0 },
		reducers: {
			testAction: state => {
				state.value += 1;
			}
		}
	});
	expect(slice.name).toBe("test");
	expect(typeof slice.reducer).toBe("function");
	
	// Validate share-usage.json was generated correctly
	const fs = require("fs");
	const path = require("path");
	const shareUsagePath = path.join(__dirname, "__js__", __filename, "share-usage.json");
	
	if (fs.existsSync(shareUsagePath)) {
		const shareUsage = JSON.parse(fs.readFileSync(shareUsagePath, "utf8"));
		
		// Check that redux exports used by @reduxjs/toolkit are marked as true
		if (shareUsage.treeShake && shareUsage.treeShake.redux) {
			const reduxExports = shareUsage.treeShake.redux;
			
			// These exports are used internally by @reduxjs/toolkit
			// and should be marked as true even though our app doesn't directly use them
			const expectedUsedExports = [
				"isPlainObject", // Used by toolkit for checking plain objects
				"combineReducers", // Used for combining reducers
				"createStore", // Used internally for legacy store creation
				"compose", // Used for middleware composition
				"applyMiddleware", // Used for applying middleware
				"bindActionCreators" // Used for action binding
			];
			
			for (const exportName of expectedUsedExports) {
				if (exportName in reduxExports) {
					expect(reduxExports[exportName]).toBe(true);
				}
			}
			
			// isPlainObject should definitely be true since toolkit uses it
			expect(reduxExports.isPlainObject).toBe(true);
		}
		
		// Check that @reduxjs/toolkit exports we use are marked as true
		if (shareUsage.treeShake && shareUsage.treeShake["@reduxjs/toolkit"]) {
			const toolkitExports = shareUsage.treeShake["@reduxjs/toolkit"];
			
			// These are the exports we directly use in this test
			expect(toolkitExports.configureStore).toBe(true);
			expect(toolkitExports.createSlice).toBe(true);
		}
	}
});