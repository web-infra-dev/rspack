import * as ns from "./lib";
export const read = () => [Object.keys(ns).sort(), ns.value(), ns.readDeferred()];
---
import * as ns from "./lib";
export const read = () => [Object.keys(ns).sort(), ns.value(), ns.readDeferred()];
---
import { local, readDeferred } from "./lib";
export const read = () => [local, readDeferred()];
