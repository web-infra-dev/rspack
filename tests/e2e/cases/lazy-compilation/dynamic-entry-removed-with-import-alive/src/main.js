document.body.id = 'main';
import('./shared.js').then(() => {
  document.body.dataset.sharedLoaded = '1';
});
