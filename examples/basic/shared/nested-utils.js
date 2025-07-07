// Nested utility functions to test PURE annotations
export const validateEmail = email => {
	const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
	return emailRegex.test(email);
};

export const generateId = () => {
	return Math.random().toString(36).substr(2, 9);
};

export const deepClone = obj => {
	if (obj === null || typeof obj !== "object") return obj;
	if (obj instanceof Date) return new Date(obj.getTime());
	if (Array.isArray(obj)) return obj.map(item => deepClone(item));
	if (typeof obj === "object") {
		const copy = {};
		for (const key of Object.keys(obj)) {
			copy[key] = deepClone(obj[key]);
		}
		return copy;
	}
};

export const sortBy = (array, key) => {
	return array.sort((a, b) => {
		if (a[key] < b[key]) return -1;
		if (a[key] > b[key]) return 1;
		return 0;
	});
};

export default {
	validateEmail,
	generateId,
	deepClone,
	sortBy
};
