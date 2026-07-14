import "./side-effect";
import { first } from "./sync-first";
import { second } from "./sync-second";
export { value } from "./leaf";

export const sync = `${first}:${second}`;
export const load = () => import(/* webpackChunkName: "next" */ "./lazy");
