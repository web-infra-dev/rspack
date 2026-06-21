export function deepMerge(state, bucket) {
	return { ...state, ...bucket };
}
