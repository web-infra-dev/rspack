import "./side-effect";
import { first } from "./sync-first";
import { second } from "./sync-second";
export { value } from "./leaf";

export const sync = `${first}:${second}`;
export const load = () =>
  Promise.all([
    import(/* webpackChunkName: "lazy" */ "./lazy"),
    import(/* webpackChunkName: "lazy" */ "./lazy"),
  ]);
