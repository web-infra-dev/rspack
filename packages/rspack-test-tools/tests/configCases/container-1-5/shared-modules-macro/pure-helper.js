// Pure helper functions that should be tree-shaken when not used
export function pureHelper() {
	return "pure helper result";
}

export function anotherPureHelper() {
	return "another pure helper result";
}

export function generateId() {
	return Math.random().toString(36).substr(2, 9);
}

export function hashString(str) {
	let hash = 0;
	for (let i = 0; i < str.length; i++) {
		const char = str.charCodeAt(i);
		hash = ((hash << 5) - hash) + char;
		hash = hash & hash; // Convert to 32bit integer
	}
	return hash.toString();
}

// Unused pure functions
export function unusedPureFunction() {
	return "unused pure function";
}