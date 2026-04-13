import * as THREE from 'three';
import { Sky } from 'three/examples/jsm/objects/Sky.js';

const SUN_ELEVATION = 20;
const SUN_AZIMUTH = 45;
const TURBIDITY = 2;
const RAYLEIGH = 1.0;
const MIE_COEFFICIENT = 0.003;
const MIE_DIRECTIONAL_G = 0.7;

function getSunPosition(): THREE.Vector3 {
  const phi = THREE.MathUtils.degToRad(90 - SUN_ELEVATION);
  const theta = THREE.MathUtils.degToRad(SUN_AZIMUTH);
  return new THREE.Vector3().setFromSphericalCoords(1, phi, theta);
}

export function createSkyObject(): Sky {
  const sky = new Sky();
  sky.scale.setScalar(10000);

  const sunPos = getSunPosition();
  const uniforms = sky.material.uniforms;
  uniforms['turbidity'].value = TURBIDITY;
  uniforms['rayleigh'].value = RAYLEIGH;
  uniforms['mieCoefficient'].value = MIE_COEFFICIENT;
  uniforms['mieDirectionalG'].value = MIE_DIRECTIONAL_G;
  uniforms['sunPosition'].value.copy(sunPos);

  const mat = sky.material as THREE.ShaderMaterial;
  mat.onBeforeCompile = (shader) => {
    shader.fragmentShader = shader.fragmentShader.replace(
      'gl_FragColor = vec4( texColor, 1.0 );',
      'gl_FragColor = vec4( texColor * 0.2, 1.0 );',
    );
  };

  return sky;
}
