import "./side-effect";
import { first } from "./sync-first";
import { second } from "./sync-second";
export { value } from "./leaf";

export const sync = `${first}:${second}`;
export const load = () =>
  Promise.all([
    import(/* webpackChunkName: "next" */ "./other"),
    import(/* webpackChunkName: "next" */ "./lazy"),
  ]);
