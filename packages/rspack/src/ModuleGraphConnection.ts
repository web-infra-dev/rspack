import { ModuleGraphConnection as BindingModuleGraphConnection } from '@rspack/binding';

export const TRANSITIVE_ONLY = Symbol('transitive only');
export const CIRCULAR_CONNECTION = Symbol('circular connection');

export type ConnectionState =
	| boolean
	| typeof TRANSITIVE_ONLY
	| typeof CIRCULAR_CONNECTION;

const originalGetActiveState =
	// biome-ignore lint/suspicious/noExplicitAny: prototype patching
	(BindingModuleGraphConnection as any).prototype.getActiveState;

// biome-ignore lint/suspicious/noExplicitAny: prototype patching
(BindingModuleGraphConnection as any).prototype.getActiveState = function (
	runtime: string | string[] | undefined,
): ConnectionState {
	const state: boolean | string = originalGetActiveState.call(this, runtime);
	if (typeof state === 'boolean') {
		return state;
	}
	if (state === 'transitive-only') {
		return TRANSITIVE_ONLY;
	}
	if (state === 'circular') {
		return CIRCULAR_CONNECTION;
	}
	return state as never;
};
