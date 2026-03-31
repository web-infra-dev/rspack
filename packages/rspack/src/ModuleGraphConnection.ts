export const TRANSITIVE_ONLY = Symbol('transitive only');
export const CIRCULAR_CONNECTION = Symbol('circular connection');

export type ConnectionState =
	| boolean
	| typeof TRANSITIVE_ONLY
	| typeof CIRCULAR_CONNECTION;
