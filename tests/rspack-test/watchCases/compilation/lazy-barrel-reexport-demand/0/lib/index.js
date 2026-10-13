import { createStore } from "./vanilla";
export * from "./vanilla";
export const create = value => createStore(value);
