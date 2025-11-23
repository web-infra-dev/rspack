import { compose, curry, filter, map, path, pipe, prop, reduce } from "ramda";

// Export functions that use only a subset of ramda
export const composeTransforms = (...fns) => compose(...fns);

export const pipeTransforms = (...fns) => pipe(...fns);

export const createCurriedFunction = fn => curry(fn);

export const mapData = (fn, data) => map(fn, data);

export const filterData = (predicate, data) => filter(predicate, data);

export const reduceData = (reducer, initial, data) =>
	reduce(reducer, initial, data);

export const getProperty = (propName, obj) => prop(propName, obj);

export const getNestedProperty = (pathArray, obj) => path(pathArray, obj);

// Example usage functions
export const processUserData = pipe(
	filter(user => user.active),
	map(user => ({
		...user,
		displayName: user.name.toUpperCase()
	}))
);

export const sumValues = reduce((acc, val) => acc + val, 0);
