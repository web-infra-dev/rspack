import type { RawSnapshotOptions, RawSnapshotStrategy } from "@rspack/binding";

export interface Snapshot {
	resolveBuildDependencies?: Partial<RawSnapshotStrategy>;
	buildDependencies?: Partial<RawSnapshotStrategy>;
	resolve?: Partial<RawSnapshotStrategy>;
	module?: Partial<RawSnapshotStrategy>;
}

export type ResolvedSnapshot = RawSnapshotOptions;

export function resolveSnapshotOptions(
	snapshot: Snapshot = {}
): ResolvedSnapshot {
	const { resolveBuildDependencies, buildDependencies, resolve, module } =
		snapshot;
	const defaultSnapshotStrategy = {
		hash: false,
		timestamp: true
	};
	return {
		resolveBuildDependencies: {
			...defaultSnapshotStrategy,
			...resolveBuildDependencies
		},
		buildDependencies: { ...defaultSnapshotStrategy, ...buildDependencies },
		resolve: { ...defaultSnapshotStrategy, ...resolve },
		module: { ...defaultSnapshotStrategy, ...module }
	};
}
