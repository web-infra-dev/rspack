import * as ns from "./lib";
export const read = () => Object.keys(ns).sort();
---
import { local } from "./lib";
export const read = () => [local];
