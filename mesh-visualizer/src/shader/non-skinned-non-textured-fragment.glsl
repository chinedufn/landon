precision mediump float;

// TODO: Pass this uniform in
uniform vec3 uCameraPos;

varying vec3 vNormal;
varying vec3 vWorldSpacePos;


// We use a gold material
void main(void) {
  vec3 lightColor = vec3(1.0, 1.0, 1.0);
  vec3 lightPos = vec3(1.0, 5.0, 3.0);

  vec3 ambient = vec3(0.24725, 0.1995, 0.0745);

  vec3 normal = normalize(vNormal);
  vec3 lightDir = normalize(lightPos - vWorldSpacePos);
  float diff = max(dot(normal, lightDir), 0.0);

  vec3 diffuse = diff * vec3(0.75164, 0.60648, 0.22648);

  float shininess = 0.4;

  vec3 viewDir = normalize(uCameraPos - vWorldSpacePos);
  vec3 reflectDir = reflect(-lightDir, normal);
  float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32.0);
  vec3 specular = shininess * spec * vec3(0.628281, 0.555802, 0.366065);

  vec4 lighting = vec4(ambient + diffuse + specular, 1.0);

   vec4 meshColor = vec4(1.0, 0.0, 0.0, 1.0);
   gl_FragColor = meshColor * lighting;
}

