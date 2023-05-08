import { Component } from '@angular/core';
import { AccordionConfig } from 'ngx-bootstrap/accordion';

// such override allows to keep some initial values

export function getAccordionConfig(): AccordionConfig {
  return Object.assign(new AccordionConfig(), { closeOthers: true });
}

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-accordion-config',
  templateUrl: './config.html',
  providers: [{ provide: AccordionConfig, useFactory: getAccordionConfig }]
})
export class DemoAccordionConfigComponent {}
