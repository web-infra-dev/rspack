import dep from './dep.cjs';
export const a = dep.value;
export const loadAsync = () => import('./async-a.js');
