const root = document.getElementById('root');
import('remote/RemoteModule').then((mod) => {
  root.textContent = mod.default();
});
