import { Component } from '@angular/core';
import { TabsetConfig } from 'ngx-bootstrap/tabs';

// such override allows to keep some initial values

export function getTabsetConfig(): TabsetConfig {
  return Object.assign(new TabsetConfig(), { type: 'pills', isKeysAllowed: true });
}

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-tabs-config',
  templateUrl: './config.html',
  providers: [{ provide: TabsetConfig, useFactory: getTabsetConfig }]
})
export class DemoTabsConfigComponent {}
