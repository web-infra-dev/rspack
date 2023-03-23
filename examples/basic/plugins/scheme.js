module.exports = class {
  constructor() {}
  apply(compiler) {
    const {  webpack } = compiler;
    const {
      NormalModule,
    } = webpack;
    compiler.hooks.thisCompilation.tap(
      'angular-compiler',
      (compilation, { normalModuleFactory }) => {
        normalModuleFactory.hooks.resolveForScheme
            .for('file')
            .tap('angular-compiler', (resourceData) => {
                resourceData.resource = require.resolve(resourceData.resource.slice(7));
                resourceData.path = resourceData.resource
                console.log(resourceData)

              return true;
            });
        // If no data is provided, the resource will be read from the filesystem
          NormalModule.getCompilationHooks(compilation)
            .readResourceForScheme.for("")
            .tap('angular-compiler', (source) => {
              return 'console.log("hello world")'
            });
  
        })
  }

  
}

// file:///something