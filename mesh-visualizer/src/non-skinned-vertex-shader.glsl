attribute vec3 aVertPos;

uniform mat4 uMVMatrix;
uniform mat4 uPMatrix;

void main (void) {
  gl_Position = uPMatrix * uMVMatrix * vec4(aVertPos, 1.0);
}

