const path = require('path')

module.exports = {
  devServer: {
    contentBase: path.resolve(__dirname, './mesh-visualizer/site-dist')
  },
  context: path.resolve(__dirname, './mesh-visualizer/site-dist'),
  entry: './bootstrap.js',
  output: {
    path: path.resolve(__dirname, 'mesh-visualizer/site-dist'),
    filename: 'bootstrap.js'
  },
  mode: 'production'
}
