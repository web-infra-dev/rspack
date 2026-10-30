export const value = (globalThis.namedGroupValue = 1);
module.hot.accept();
---
export const value = (globalThis.namedGroupValue = require('./dep'));
module.hot.accept();
