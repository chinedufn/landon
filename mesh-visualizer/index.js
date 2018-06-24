import { App } from './mesh_visualizer';

const app = App.new()

app.start()

export function download_model (modelName, cb) {
  const request = new window.Request(modelName)
  window.fetch(request).then(response => {
    response.text().then(modelJSONString => {
      cb(modelJSONString)
    })
  })
}

window.requestAnimationFrame(() => {
  app.draw()
  window.requestAnimationFrame(app.draw)
})
