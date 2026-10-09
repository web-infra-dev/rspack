import { policy } from "./hooks";
export const value = "v1-" + policy;
module.hot.accept();
---
export const value = "v2";
module.hot.accept();
---
import { policy } from "./hooks";
export const value = "v3-" + policy;
module.hot.accept();
---
import { policy } from "./hooks";
export const value = "v3-" + policy;
module.hot.accept();
