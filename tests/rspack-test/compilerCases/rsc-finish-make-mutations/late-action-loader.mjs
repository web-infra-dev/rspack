export default function loader(source) {
  if (!this.injectAction) return source;
  return `${source}
import { lateAction } from './late-action.js';
globalThis.lateStub = lateAction;
`;
}
