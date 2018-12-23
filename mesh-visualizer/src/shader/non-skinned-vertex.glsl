attribute vec3 aVertexPosition;
attribute vec3 aVertexNormal;
attribute vec2 aTextureCoord;

uniform mat4 perspective;
uniform mat4 view;
uniform mat4 model;

varying vec3 vNormal;
varying vec3 vWorldSpacePos;

varying vec2 vTextureCoord;

void main (void) {
  gl_Position = perspective * view * model * vec4(aVertexPosition, 1.0);

  vNormal = aVertexNormal;

  // World space is same as model space since model matrix is identity.
  // If that changes simply multiple `aVertexPos` by the model matrix.
  vWorldSpacePos = aVertexPosition;

  vTextureCoord = aTextureCoord;
}
