import { App } from './mesh_visualizer';

const app = App.new()

app.start()

export function download_meshes (cb) {
  const request = new window.Request("/dist/meshes.json")
  window.fetch(request).then(response => {
    response.text().then(meshesJson => {
      cb(meshesJson)
    })
  })
}

const image = new window.Image()
image.onload = function () {
  app.set_texture(image)
}
image.src = 'dist/textured_cube-uv-layout.png'

const draw = () => {
  app.draw()

  window.requestAnimationFrame(() => {
    draw()
  })
}
draw()
