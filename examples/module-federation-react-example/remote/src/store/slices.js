import { createSlice } from "@reduxjs/toolkit";

// Shared notification slice
export const notificationSlice = createSlice({
	name: "notifications",
	initialState: {
		messages: []
	},
	reducers: {
		addNotification: (state, action) => {
			state.messages.push({
				id: Date.now(),
				...action.payload,
				timestamp: new Date().toISOString()
			});
		},
		removeNotification: (state, action) => {
			state.messages = state.messages.filter(msg => msg.id !== action.payload);
		},
		clearNotifications: state => {
			state.messages = [];
		}
	}
});

// Shared theme slice
export const themeSlice = createSlice({
	name: "theme",
	initialState: {
		mode: "light",
		primaryColor: "#1890ff"
	},
	reducers: {
		toggleTheme: state => {
			state.mode = state.mode === "light" ? "dark" : "light";
		},
		setPrimaryColor: (state, action) => {
			state.primaryColor = action.payload;
		}
	}
});

export const { addNotification, removeNotification, clearNotifications } =
	notificationSlice.actions;
export const { toggleTheme, setPrimaryColor } = themeSlice.actions;

export default {
	notifications: notificationSlice.reducer,
	theme: themeSlice.reducer
};
