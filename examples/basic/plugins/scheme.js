const path = require('path')
module.exports = class T {
  constructor() {}
  apply(compiler) {
    compiler.hooks.thisCompilation.tap("angular-compiler", (compilation, { normalModuleFactory}) => {
      normalModuleFactory.hooks.resolveForScheme.for('file').tap('angular-compiler', (data) => {
        let resource = data.resource.replace('file://', '');
        data.resource = resource;
        data.path = resource;
      })
		});

    
  }
}