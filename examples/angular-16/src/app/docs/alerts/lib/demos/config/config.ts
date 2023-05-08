import { Component } from '@angular/core';
import { AlertConfig } from 'ngx-bootstrap/alert';

// such override allows to keep some initial values

export function getAlertConfig(): AlertConfig {
  return Object.assign(new AlertConfig(), { type: 'success' });
}

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-alert-config',
  templateUrl: './config.html',
  providers: [{ provide: AlertConfig, useFactory: getAlertConfig }]
})
export class DemoAlertConfigComponent {}
