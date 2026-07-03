import dep from './dep.cjs';
export const b = dep.value;
export const loadAsync = () => import('./async-b.js');
