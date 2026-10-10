module.hot.accept();
export const value = 'before';
---
module.hot.accept();
export { value } from './dependency';
