import {
  Object3D,
  Camera,
  Vector3,
  Euler,
  MOUSE
} from '../../../src/Three';

export class TransformControls extends Object3D {
  constructor(object: Camera, domElement?: HTMLElement);

  domElement: HTMLElement;

  // API

  camera: Camera;
  object: Object3D;
  enabled: boolean;
  axis: string;
  mode: string;
  translationSnap: Vector3;
  rotationSnap: Vector3;
  space: string;
  size: number;
  dragging: boolean;
  showX: boolean;
  showY: boolean;
  showZ: boolean;
  isTransformControls: boolean;
  visible: boolean;
  mouseButtons: {
    LEFT: MOUSE; 
    MIDDLE: MOUSE;
    RIGHT: MOUSE;
  };

  attach(object: Object3D): this;
  detach(): this;
  pointerHover(pointer: Object): void;
  pointerDown(pointer: Object): void;
  pointerMove(pointer: Object): void;
  pointerUp(pointer: Object): void;
  getMode(): string;
  setMode(mode: string): void;
  setTranslationSnap(translationSnap: Number | null): void;
  setRotationSnap(rotationSnap: Number | null): void;
  setSize(size: number): void;
  setSpace(space: string): void;
  dispose(): void;
  update(): void;

}
