import { createSlice } from "@reduxjs/toolkit";

// Create a simple counter slice
export const counterSlice = createSlice({
	name: "counter",
	initialState: {
		value: 0
	},
	reducers: {
		increment: state => {
			state.value += 1;
		},
		decrement: state => {
			state.value -= 1;
		},
		incrementByAmount: (state, action) => {
			state.value += action.payload;
		}
	}
});

export const { increment, decrement, incrementByAmount } = counterSlice.actions;
export const counterReducer = counterSlice.reducer;
