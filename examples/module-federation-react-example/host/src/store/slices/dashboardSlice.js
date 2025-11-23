import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { delay } from "lodash-es";

// Simulate API call
export const fetchDashboardStats = createAsyncThunk(
	"dashboard/fetchStats",
	async () => {
		await delay(1000);
		return {
			totalUsers: 12543,
			activeUsers: 8921,
			revenue: 458320,
			growth: 12.5
		};
	}
);

const dashboardSlice = createSlice({
	name: "dashboard",
	initialState: {
		stats: null,
		loading: false,
		error: null
	},
	reducers: {
		resetDashboard: state => {
			state.stats = null;
			state.loading = false;
			state.error = null;
		}
	},
	extraReducers: builder => {
		builder
			.addCase(fetchDashboardStats.pending, state => {
				state.loading = true;
				state.error = null;
			})
			.addCase(fetchDashboardStats.fulfilled, (state, action) => {
				state.loading = false;
				state.stats = action.payload;
			})
			.addCase(fetchDashboardStats.rejected, (state, action) => {
				state.loading = false;
				state.error = action.error.message;
			});
	}
});

export const { resetDashboard } = dashboardSlice.actions;
export default dashboardSlice.reducer;
