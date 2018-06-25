attribute vec3 aVertPos;
attribute vec3 aVertNormal;

uniform mat4 uMVMatrix;
uniform mat4 uPMatrix;

varying vec3 vNormal;
varying vec3 vWorldSpacePos;

void main (void) {
  gl_Position = uPMatrix * uMVMatrix * vec4(aVertPos, 1.0);

  vNormal = aVertNormal;

  // World space is same as model space since model matrix is identity.
  // If that changes simply multiple `aVertexPos` by the model matrix.
  vWorldSpacePos = aVertPos;
}

