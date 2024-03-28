import { rspack as rspackFn } from "./rspack";
import * as rspackExports from "./exports";
// add exports on rspack() function
type Rspack = typeof rspackFn &
	typeof rspackExports & { rspack: Rspack; webpack: Rspack };
const fn: any = Object.assign(rspackFn, rspackExports);
fn.rspack = fn;
fn.webpack = fn;
const rspack: Rspack = fn;

export * from "./exports";
export { rspack };
export default rspack;
