import { resource, createResource } from "./disposable";

using a = resource;
using b = /*#__PURE__*/ createResource("sync");

export const value = 1;
