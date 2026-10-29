export function resolveTarget() {
	return import.meta.resolve('./target');
}

export function resolveConditional(useTarget) {
	return import.meta.resolve(useTarget ? './target' : './nested/resolver');
}

export function getResolveType() {
	return typeof import.meta.resolve;
}
