import { createStore } from "./vanilla";
export * from "./vanilla";
export { createStore as makeStore } from "./vanilla";
export const create = value => createStore(value);
