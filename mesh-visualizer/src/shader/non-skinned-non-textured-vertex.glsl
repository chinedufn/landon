attribute vec3 aVertexPosition;
attribute vec3 aVertexNormal;

uniform mat4 perspective;
uniform mat4 view;
uniform mat4 model;

varying vec3 vNormal;
varying vec3 vWorldSpacePos;

varying vec2 vTextureCoord;

void main (void) {
  vec4 vertexWorldPos = model * vec4(aVertexPosition, 1.0);

  gl_Position = perspective * view * vertexWorldPos;

  // FIXME: Transform normal using model matrix ..
  vNormal = aVertexNormal;

  vWorldSpacePos = vertexWorldPos.xyz;
}
