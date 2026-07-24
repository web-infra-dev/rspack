export const unused = {
	id:
		(globalThis.unusedObjectInitializerRuns =
			(globalThis.unusedObjectInitializerRuns || 0) + 1),
	loader: () =>
		import(/* webpackChunkName: "unused" */ "./unused")
};

export const loaders = [...(FEATURE_ENABLED ? [unused] : [])];
