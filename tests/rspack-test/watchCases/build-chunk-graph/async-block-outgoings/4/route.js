import "./side-effect";
import { first } from "./sync-first";
import { second } from "./sync-second";
export { value } from "./leaf";

const beforeAsyncEdges = "moved-without-changing-topology";
export const sync = `${first}:${second}`;
export const load = () => {
  void beforeAsyncEdges;
  return Promise.all([
    import(/* webpackChunkName: "next" */ "./other"),
    import(/* webpackChunkName: "next" */ "./lazy"),
  ]);
};
