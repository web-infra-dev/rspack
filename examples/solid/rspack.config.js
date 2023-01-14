module.exports = {
  context: __dirname,
  entry: {
    main: './src/index.jsx'
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        type: 'css'
      },
      {
        test: /\.module.css$/,
        type: 'css/module'
      },
      {
        test: /\.jsx$/,
        use: [
          {
            loader: 'babel-loader',
            options: {
              presets: [
                ["solid"]
              ]
            }
          }
        ]
      }
    ]
  },
  builtins: {
    html: [{
      template: './index.html'
    }],
    react: {
      importSource: 'solid-js'
    }
  }
}