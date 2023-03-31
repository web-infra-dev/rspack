// TODO(ahabhgk): needs WorkerDependency
module.exports = {
  module: {
    rules: [
      {
        dependency: "url",
        type: 'js',
      }
    ]
  },
  resolve: {
    byDependency: {
      url: {
        extensions: ['.js']
      }
    }
  }
}
