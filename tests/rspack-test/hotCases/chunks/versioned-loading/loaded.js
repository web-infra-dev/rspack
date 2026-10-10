export { value } from './before';
module.hot.accept();
---
export { value } from './after';
module.hot.accept();
---
export { value } from './latest';
module.hot.accept();
