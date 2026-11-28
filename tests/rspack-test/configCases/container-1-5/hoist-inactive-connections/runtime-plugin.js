import { pruned } from "pruned-pkg";
import { runtimeSpecific } from "runtime-specific-pkg";
import { used } from "used-pkg";

export function getRuntimeSpecificValue() {
	return runtimeSpecific;
}

export default function () {
	return {
		name: "hoist-inactive-connections",
		beforeInit(args) {
			globalThis.hoistInactiveConnectionsUsed = used;
			return args;
		}
	};
}
