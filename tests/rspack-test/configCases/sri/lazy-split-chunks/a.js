import { lib } from './lib';
export const load = () => import(/* webpackChunkName: "leaf" */ './leaf');
export const value = lib('a');
