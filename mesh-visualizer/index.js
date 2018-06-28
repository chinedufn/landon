import { App } from './mesh_visualizer';

const app = App.new()

app.start()

export function download_mesh (meshName, meshURL, cb) {
  const request = new window.Request(meshURL)
  window.fetch(request).then(response => {
    response.text().then(modelJSONString => {
      cb(meshName, modelJSONString)
    })
  })
}

const draw = () => {
  app.draw()

  window.requestAnimationFrame(() => {
    draw()
  })
}
draw()
