import { deepMerge } from "./merge";

export function reducerFactory(prefix) {
	return state => deepMerge(state, { type: prefix });
}

export function actionFactory(prefix) {
	globalThis.__pureFunctionsSeveredExportActionFactoryCalls =
		(globalThis.__pureFunctionsSeveredExportActionFactoryCalls || 0) + 1;
	return bucket => ({ type: prefix, bucket });
}

export const pureDrop = () => "SHOULD_BE_DROPPED";
