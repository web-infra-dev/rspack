import path from 'node:path';
export default function(code) {
  for (const file of ['theme/base.less', 'theme/colors.less', 'node_modules/exported/package.json', 'node_modules/exported/nested.less', 'modules/custom-package/package.json']) expect(this.getDependencies()).toContain(path.join(this.context, file));
  return code;
}
