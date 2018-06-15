const path = require('path')

module.exports = {
  devServer: {
    contentBase: path.resolve(__dirname, './mesh-visualizer')
  },
  context: path.resolve(__dirname, './mesh-visualizer'),
  entry: './bootstrap.js',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'bootstrap.js'
  },
  mode: 'development'
}
