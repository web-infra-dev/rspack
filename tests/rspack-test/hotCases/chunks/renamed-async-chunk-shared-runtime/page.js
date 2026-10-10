import "./hooks";
import "./shared/common";
export const value = 1;
module.hot.accept();
---
import "./shared/common";
export const value = 2;
module.hot.accept();
---
import "./shared/common";
export const value = 3;
module.hot.accept();
