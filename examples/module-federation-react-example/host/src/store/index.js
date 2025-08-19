import { configureStore } from "@reduxjs/toolkit";
import analyticsReducer from "./slices/analyticsSlice.js";
import dashboardReducer from "./slices/dashboardSlice.js";
import usersReducer from "./slices/usersSlice.js";

export const store = configureStore({
	reducer: {
		dashboard: dashboardReducer,
		users: usersReducer,
		analytics: analyticsReducer
	}
});
