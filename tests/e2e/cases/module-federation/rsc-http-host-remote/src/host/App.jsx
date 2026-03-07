import { RemoteWidget } from 'rscRemote/RemoteWidget';
import { readLayerInfo } from 'rscRemote/LayerInfo';

export function App() {
  return (
    <main>
      <h1 data-testid="title">host-rsc-mf-e2e</h1>
      <div data-testid="layer-info">{readLayerInfo()}</div>
      <RemoteWidget />
    </main>
  );
}
