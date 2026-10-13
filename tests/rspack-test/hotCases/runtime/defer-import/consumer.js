export const getNamespaces = () => [];
---
import defer * as namespace from "./deferred";
import defer * as pending from "./pending";

export const getNamespaces = () => [namespace, pending];
---
import defer * as namespace from "./deferred";
import defer * as pending from "./pending";

export const getNamespaces = () => [namespace, pending];
// Adding async support changes the deferred namespace runtime module as well.
export const loadAsync = () => import("./async");
