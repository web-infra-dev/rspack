import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-accordion-disabled',
  templateUrl: './disabled.html'
})
export class DemoAccordionDisabledComponent {
  isFirstDisabled = false;
}
