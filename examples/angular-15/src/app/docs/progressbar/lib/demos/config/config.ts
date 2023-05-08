import { Component } from '@angular/core';
import { ProgressbarConfig } from 'ngx-bootstrap/progressbar';

// such override allows to keep some initial values

export function getProgressbarConfig(): ProgressbarConfig {
  return Object.assign(new ProgressbarConfig(), { animate: true, striped: true,  max: 150 });
}

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-progressbar-config',
  templateUrl: './config.html',
  providers: [{ provide: ProgressbarConfig, useFactory: getProgressbarConfig }]
})
export class DemoProgressbarConfigComponent {}
