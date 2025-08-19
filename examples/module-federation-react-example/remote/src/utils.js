import { debounce, omit, pick, throttle } from "lodash-es";

export const createDebouncedFunction = (fn, delay = 300) => {
	return debounce(fn, delay);
};

export const createThrottledFunction = (fn, delay = 100) => {
	return throttle(fn, delay);
};

export const pickFields = (obj, fields) => {
	return pick(obj, fields);
};

export const omitFields = (obj, fields) => {
	return omit(obj, fields);
};

export const formatUserData = userData => {
	const publicFields = pickFields(userData, ["name", "email", "role"]);
	return {
		...publicFields,
		displayName: publicFields.name
			? publicFields.name.toUpperCase()
			: "Anonymous"
	};
};
