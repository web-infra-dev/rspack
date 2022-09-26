import {
  Camera,
  EventDispatcher,
  Vector3
} from '../../../src/Three';

export class PointerLockControls extends EventDispatcher {
  constructor(camera: Camera, domElement?: HTMLElement);

  domElement: HTMLElement;
  object: Camera;

  // API

  isLocked: boolean;

  connect(): void;
  disconnect(): void;
  dispose(): void;
  getObject(): Camera;
  getDirection(v: Vector3): Vector3;
  lock(): void;
  unlock(): void;

}
