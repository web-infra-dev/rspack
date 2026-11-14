import '../setup.mjs';
export { value, update, value as default, 'x-y' as quoted } from '../ext.mjs';
import './b.mjs';
globalThis.order.push('a');
