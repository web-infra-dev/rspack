export { value } from './first.mjs';
import './second.mjs';

export const url = () => new URL('./note.txt', import.meta.url);
