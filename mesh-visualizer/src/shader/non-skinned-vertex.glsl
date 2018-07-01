attribute vec3 aVertexPosition;
attribute vec3 aVertexNormal;

uniform mat4 uMVMatrix;
uniform mat4 uPMatrix;

varying vec3 vNormal;
varying vec3 vWorldSpacePos;

void main (void) {
  gl_Position = uPMatrix * uMVMatrix * vec4(aVertexPosition, 1.0);

  vNormal = aVertexNormal;

  // World space is same as model space since model matrix is identity.
  // If that changes simply multiple `aVertexPos` by the model matrix.
  vWorldSpacePos = aVertexPosition;
}
