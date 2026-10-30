import normal from './style.less?true';
import disabled from './style.less?false';
import only from './style.less?only';
it('supports native-first, disabled and resolver-only import modes', () => {
  expect(normal).toContain('color: red;');
  expect(normal).toContain('width: 7px;');
  expect(disabled).toContain('color: red;');
  expect(disabled).not.toContain('width:');
  expect(only).toContain('color: blue;');
  expect(only).toContain('width: 7px;');
});
