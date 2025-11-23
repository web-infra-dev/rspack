it("should track inter-shared-library dependencies", async () => {
	// Import @reduxjs/toolkit which internally uses redux exports
	const toolkit = await import("@reduxjs/toolkit");
	const store = await import("./store.js");

	// Use configureStore from toolkit (which internally uses redux functions)
	const myStore = toolkit.configureStore({
		reducer: {
			counter: store.counterReducer
		},
		middleware: [],
		enhancers: []
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
	expect(typeof slice.actions.testAction).toBe("function");

	// Test createAction
	const action = toolkit.createAction("test/action");
	expect(typeof action).toBe("function");
	expect(action.type).toBe("test/action");
});
