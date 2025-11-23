import { createAsyncThunk, createSlice } from "@reduxjs/toolkit";
import { delay, random } from "lodash-es";

// Simulate API call
export const fetchAnalyticsData = createAsyncThunk(
	"analytics/fetchData",
	async () => {
		await delay(600);

		const months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];
		const generateData = () => months.map(() => random(100, 500));

		return {
			revenue: {
				labels: months,
				datasets: [
					{
						label: "Revenue",
						data: generateData(),
						borderColor: "rgb(75, 192, 192)",
						backgroundColor: "rgba(75, 192, 192, 0.2)"
					}
				]
			},
			userGrowth: {
				labels: months,
				datasets: [
					{
						label: "New Users",
						data: generateData(),
						borderColor: "rgb(54, 162, 235)",
						backgroundColor: "rgba(54, 162, 235, 0.2)"
					}
				]
			},
			categories: {
				labels: ["Desktop", "Mobile", "Tablet"],
				datasets: [
					{
						data: [random(30, 50), random(30, 50), random(10, 20)],
						backgroundColor: [
							"rgba(255, 99, 132, 0.6)",
							"rgba(54, 162, 235, 0.6)",
							"rgba(255, 206, 86, 0.6)"
						]
					}
				]
			}
		};
	}
);

const analyticsSlice = createSlice({
	name: "analytics",
	initialState: {
		data: null,
		loading: false,
		error: null,
		timeRange: "6months"
	},
	reducers: {
		setTimeRange: (state, action) => {
			state.timeRange = action.payload;
		}
	},
	extraReducers: builder => {
		builder
			.addCase(fetchAnalyticsData.pending, state => {
				state.loading = true;
				state.error = null;
			})
			.addCase(fetchAnalyticsData.fulfilled, (state, action) => {
				state.loading = false;
				state.data = action.payload;
			})
			.addCase(fetchAnalyticsData.rejected, (state, action) => {
				state.loading = false;
				state.error = action.error.message;
			});
	}
});

export const { setTimeRange } = analyticsSlice.actions;
export default analyticsSlice.reducer;
