import { foo, sideEffect } from "./dep";

export const a = foo;
export const b = sideEffect(a);
export const c = sideEffect(function () {});
