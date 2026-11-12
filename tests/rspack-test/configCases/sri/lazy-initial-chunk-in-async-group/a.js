import { lib } from './lib';
export const value = lib('a');
export const load = () => import(/* webpackChunkName: "b" */ './b');
