import { local } from './exposed';
import value from 'remote-only';
export { value };
export const readLocal = () => local;
