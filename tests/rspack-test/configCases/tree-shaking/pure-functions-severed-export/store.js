import { actionFactory, reducerFactory } from "./factories";
import { pureDrop } from "./factories";

export const mergeStore = actionFactory("Store/MERGE");

export const droppedPure = pureDrop();

export const storeReducers = {
	store: reducerFactory("Store/MERGE")
};
