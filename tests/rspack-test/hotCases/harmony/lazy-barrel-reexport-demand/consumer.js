import * as ns from "./lib";
import * as deferred from "./lib/deferred";
export const read = () => [
	Object.keys(ns).sort(), Object.keys(deferred).sort(),
	ns.value(), deferred.value(), deferred.readDeferred()
];
---
import * as ns from "./lib";
import * as deferred from "./lib/deferred";
export const read = () => [
	Object.keys(ns).sort(), Object.keys(deferred).sort(),
	ns.value(), deferred.value(), deferred.readDeferred()
];
---
import { local } from "./lib";
import { local as deferredLocal, readDeferred } from "./lib/deferred";
export const read = () => [local, deferredLocal, readDeferred()];
