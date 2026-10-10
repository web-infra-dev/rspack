import "./hooks";
export const value = 1;
module.hot.accept();
---
export const value = 2;
module.hot.accept();
---
export const value = 3;
module.hot.accept();
