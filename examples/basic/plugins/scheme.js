module.exports = class T {
  constructor() {}
  apply(compiler) {
    compiler.hooks.thisCompilation.tap("angular-compiler", (compilation, { normalModuleFactory}) => {
      console.log(normalModuleFactory)
      normalModuleFactory.hooks.resolveForScheme.for('test').tap('angular-compiler', (data) => {

        console.log(data)

      })
		});

    
  }
}