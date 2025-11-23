import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { delay, sortBy } from "lodash-es";

// Simulate API call
export const fetchUsers = createAsyncThunk("users/fetchUsers", async () => {
	await delay(800);
	return [
		{
			id: 1,
			name: "John Doe",
			email: "john@example.com",
			role: "Admin",
			status: "active"
		},
		{
			id: 2,
			name: "Jane Smith",
			email: "jane@example.com",
			role: "User",
			status: "active"
		},
		{
			id: 3,
			name: "Bob Johnson",
			email: "bob@example.com",
			role: "User",
			status: "inactive"
		},
		{
			id: 4,
			name: "Alice Brown",
			email: "alice@example.com",
			role: "Manager",
			status: "active"
		},
		{
			id: 5,
			name: "Charlie Wilson",
			email: "charlie@example.com",
			role: "User",
			status: "active"
		}
	];
});

const usersSlice = createSlice({
	name: "users",
	initialState: {
		list: [],
		loading: false,
		error: null,
		sortField: "name"
	},
	reducers: {
		sortUsers: (state, action) => {
			state.sortField = action.payload;
			state.list = sortBy(state.list, [action.payload]);
		},
		updateUserStatus: (state, action) => {
			const user = state.list.find(u => u.id === action.payload.id);
			if (user) {
				user.status = action.payload.status;
			}
		}
	},
	extraReducers: builder => {
		builder
			.addCase(fetchUsers.pending, state => {
				state.loading = true;
				state.error = null;
			})
			.addCase(fetchUsers.fulfilled, (state, action) => {
				state.loading = false;
				state.list = sortBy(action.payload, [state.sortField]);
			})
			.addCase(fetchUsers.rejected, (state, action) => {
				state.loading = false;
				state.error = action.error.message;
			});
	}
});

export const { sortUsers, updateUserStatus } = usersSlice.actions;
export default usersSlice.reducer;
