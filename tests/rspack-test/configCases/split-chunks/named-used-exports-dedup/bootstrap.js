import { init } from './modules/singleton';
import { helper } from './modules/helper';
import './prelude';

init();

it('should share the initialized singleton with the extracted helper', () => {
  expect(helper()).toBe(true);
});
