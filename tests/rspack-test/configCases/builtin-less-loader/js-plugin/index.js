import first from './first.less';
import second from './second.less';

it('runs JavaScript plugins with Less scope and isolated compilation state', () => {
  expect(first).toBe('.first {\n  width: 13px;\n}\n');
  expect(second).toBe('.second {\n  width: 11px;\n}\n');
});
