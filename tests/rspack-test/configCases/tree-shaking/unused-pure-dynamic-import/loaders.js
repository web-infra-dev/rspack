export const unused = {
	id:
		(globalThis.unusedObjectInitializerRuns =
			(globalThis.unusedObjectInitializerRuns || 0) + 1),
	loader: () =>
		import(
			/* webpackChunkName: "unused-dynamic-import" */
			"./unused"
		)
};

export const usedAtInitialization = {
	loader: import(
		/* webpackChunkName: "used-dynamic-import" */
		"./used"
	)
};

export const loaders = [...(FEATURE_ENABLED ? [unused] : [])];
