import './red.css';

if (new URL(window.location.href).search) {
  // @ts-expect-error change.js no dts
  import('./change.js');
}
