export const load = () => import(/* webpackChunkName: 'page' */ './page');
export const loadAgain = () => import(/* webpackChunkName: 'page' */ './page');
module.hot.accept();
---
// Moving the imports must not change the identity of the named group.
export const load = () => import(/* webpackChunkName: 'page' */ './page');
export const loadAgain = () => import(/* webpackChunkName: 'page' */ './page');
module.hot.accept();
