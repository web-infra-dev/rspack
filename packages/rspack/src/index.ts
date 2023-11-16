import { rspack as rspackFn } from "./rspack";
import * as rspackExports from "./exports";

// add exports on rspack() function
const rspack = Object.assign(rspackFn, rspackExports);
export { rspack };
export { rspack as webpack };
export default rspack;

// re-export, so user can use named import
export * from "./exports";
