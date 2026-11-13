export { token as nested } from './middle.mjs';
export {
  default as extDefault,
  named,
  '你好' as nihao,
  'default' as reservedDefault,
} from './ext.mjs';
import './side.mjs';

expect(globalThis.__order).toEqual(['ext', 'side']);
