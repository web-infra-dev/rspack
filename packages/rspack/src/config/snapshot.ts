import type { RawSnapshotOptions, RawSnapshotStrategy } from "@rspack/binding";

export interface Snapshot {
	resolve?: Partial<RawSnapshotStrategy>;
	module?: Partial<RawSnapshotStrategy>;
}

export type ResolvedSnapshot = RawSnapshotOptions;

export function resolveSnapshotOptions(
	snapshot: Snapshot = {}
): ResolvedSnapshot {
	const { resolve, module } = snapshot;
	const defaultSnapshotStrategy = {
		hash: false,
		timestamp: true
	};
	return {
		resolve: { ...defaultSnapshotStrategy, ...resolve },
		module: { ...defaultSnapshotStrategy, ...module }
	};
}
