import { useEffect, useRef } from 'react';
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { EffectComposer } from 'three/examples/jsm/postprocessing/EffectComposer.js';
import { RenderPass } from 'three/examples/jsm/postprocessing/RenderPass.js';
import { GTAOPass } from 'three/examples/jsm/postprocessing/GTAOPass.js';
import { OutputPass } from 'three/examples/jsm/postprocessing/OutputPass.js';
import { createSkyObject } from './sky';

export interface SceneRefs {
  container: React.RefObject<HTMLDivElement | null>;
  scene: React.RefObject<THREE.Scene | null>;
  camera: React.RefObject<THREE.PerspectiveCamera | null>;
  controls: React.RefObject<OrbitControls | null>;
  renderer: React.RefObject<THREE.WebGLRenderer | null>;
  frame: React.RefObject<number>;
}

export function useThreeScene(onAnimate?: (scene: THREE.Scene) => void): SceneRefs {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const sceneRef = useRef<THREE.Scene | null>(null);
  const cameraRef = useRef<THREE.PerspectiveCamera | null>(null);
  const controlsRef = useRef<OrbitControls | null>(null);
  const rendererRef = useRef<THREE.WebGLRenderer | null>(null);
  const frameRef = useRef<number>(0);
  const onAnimateRef = useRef(onAnimate);
  onAnimateRef.current = onAnimate;

  useEffect(() => {
    if (!containerRef.current) return;
    const container = containerRef.current;

    const scene = new THREE.Scene();
    sceneRef.current = scene;

    const sky = createSkyObject();
    scene.add(sky);

    const rect = container.getBoundingClientRect();
    const w = rect.width || 500;
    const h = rect.height || 500;

    const camera = new THREE.PerspectiveCamera(45, w / h, 0.1, 10000);
    camera.position.set(200, 150, 200);
    cameraRef.current = camera;

    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(w, h);
    renderer.setPixelRatio(window.devicePixelRatio);
    renderer.toneMapping = THREE.ACESFilmicToneMapping;
    renderer.toneMappingExposure = 0.8;
    renderer.shadowMap.enabled = true;
    renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    container.appendChild(renderer.domElement);
    rendererRef.current = renderer;

    // -- Post-processing --
    const composer = new EffectComposer(renderer);
    const renderPass = new RenderPass(scene, camera);
    composer.addPass(renderPass);

    const gtaoPass = new GTAOPass(scene, camera, w, h, {
      radius: 50,
      distanceExponent: 2,
      thickness: 20,
      scale: 1,
      distanceFallOff: 1,
    });
    gtaoPass.blendIntensity = 1.5;
    composer.addPass(gtaoPass);

    const outputPass = new OutputPass();
    composer.addPass(outputPass);

    // -- Controls --
    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.dampingFactor = 0.05;
    controlsRef.current = controls;

    // -- Lighting --
    const sun = new THREE.DirectionalLight(0xffe0b2, 1.5);
    sun.position.set(250, 500, -400);
    sun.castShadow = true;
    sun.shadow.mapSize.width = 2048;
    sun.shadow.mapSize.height = 2048;
    sun.shadow.camera.near = 1;
    sun.shadow.camera.far = 2000;
    sun.shadow.camera.left = -500;
    sun.shadow.camera.right = 500;
    sun.shadow.camera.top = 500;
    sun.shadow.camera.bottom = -500;
    sun.shadow.bias = -0.002;
    sun.shadow.normalBias = 0.02;
    scene.add(sun);

    const fillLight = new THREE.DirectionalLight(0x6baed6, 0.5);
    fillLight.position.set(-100, 50, -100);
    scene.add(fillLight);

    const rimLight = new THREE.DirectionalLight(0xffffff, 0.2);
    rimLight.position.set(0, 100, -200);
    scene.add(rimLight);

    const ambientLight = new THREE.AmbientLight(0xffffff, 0.4);
    scene.add(ambientLight);

    // -- Ground plane for shadow receiving --
    const groundGeom = new THREE.PlaneGeometry(2000, 2000);
    groundGeom.rotateX(-Math.PI / 2);
    const groundMat = new THREE.ShadowMaterial({ opacity: 0.2 });
    const ground = new THREE.Mesh(groundGeom, groundMat);
    ground.name = '__ground_shadow';
    ground.receiveShadow = true;
    scene.add(ground);

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const origRenderOverride = (gtaoPass as any)._renderOverride.bind(gtaoPass);
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (gtaoPass as any)._renderOverride = (...args: any[]) => {
      ground.visible = false;
      origRenderOverride(...args);
      ground.visible = true;
    };

    // -- Render loop --
    const animate = () => {
      frameRef.current = requestAnimationFrame(animate);
      onAnimateRef.current?.(scene);
      controls.update();
      composer.render();
    };
    animate();

    const handleResize = () => {
      const r = container.getBoundingClientRect();
      if (r.width > 0 && r.height > 0) {
        camera.aspect = r.width / r.height;
        camera.updateProjectionMatrix();
        renderer.setSize(r.width, r.height);
        composer.setSize(r.width, r.height);
      }
    };
    const resizeObserver = new ResizeObserver(handleResize);
    resizeObserver.observe(container);

    return () => {
      resizeObserver.disconnect();
      cancelAnimationFrame(frameRef.current);
      composer.dispose();
      renderer.dispose();
      if (renderer.domElement && container.contains(renderer.domElement)) {
        container.removeChild(renderer.domElement);
      }
    };
  }, []);

  return {
    container: containerRef,
    scene: sceneRef,
    camera: cameraRef,
    controls: controlsRef,
    renderer: rendererRef,
    frame: frameRef,
  };
}

export function clearSceneModels(scene: THREE.Scene) {
  const toRemove = scene.children.filter(
    (c) => !(c instanceof THREE.Light) && c.name !== '__ground_shadow' && !('isSky' in c),
  );
  toRemove.forEach((c) => scene.remove(c));

  const shadow = scene.getObjectByName('__ground_shadow');
  if (shadow) {
    shadow.position.set(0, 0, 0);
  }
}

export function frameBounds(
  camera: THREE.PerspectiveCamera,
  controls: OrbitControls,
  scene: THREE.Scene,
  target: THREE.Object3D,
) {
  const box = new THREE.Box3().setFromObject(target);
  const center = box.getCenter(new THREE.Vector3());
  const size = box.getSize(new THREE.Vector3());
  const maxDim = Math.max(size.x, size.y, size.z);
  const distance = maxDim * 1.5;

  camera.position.set(
    center.x + distance * 0.15,
    center.y + distance * 0.3,
    center.z - distance * 1.2,
  );
  controls.target.copy(center);
  controls.update();

  const ground = scene.getObjectByName('__ground_shadow');
  if (ground) {
    ground.position.set(center.x, center.y - size.y * 0.5 - 0.5, center.z);
  }

  const sun = scene.children.find(
    (c): c is THREE.DirectionalLight => c instanceof THREE.DirectionalLight && c.castShadow,
  );
  if (sun) {
    sun.position.set(center.x + 250, center.y + 500, center.z - 400);
    sun.target.position.copy(center);
    scene.add(sun.target);
    const halfExtent = maxDim * 0.8;
    sun.shadow.camera.left = -halfExtent;
    sun.shadow.camera.right = halfExtent;
    sun.shadow.camera.top = halfExtent;
    sun.shadow.camera.bottom = -halfExtent;
    sun.shadow.camera.near = 1;
    sun.shadow.camera.far = 2000;
    sun.shadow.camera.updateProjectionMatrix();
  }
}
