import state from "./state.js";

state.first = await new Promise((resolve) => {
	// Slower than the other entry, so awaiting only the last one fails here.
	setTimeout(() => resolve("first"), 100);
});
