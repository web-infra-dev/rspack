export const load = () => import(/* webpackChunkName: "page" */ "./pages/page");
export const loadContext = name => import(`./pages/${name}.js`);
export const loadLate = () => import(/* webpackChunkName: "late" */ "./late");
