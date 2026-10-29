import path from 'node:path';

export default function (code) {
  expect(this.getDependencies()).toContain(path.join(this.context, 'variables.less'));
  expect(this.getDependencies()).toContain(path.join(this.context, 'icon.svg'));
  expect(this.getMissingDependencies()).toContain(path.join(this.context, 'optional.less'));
  return code;
}
