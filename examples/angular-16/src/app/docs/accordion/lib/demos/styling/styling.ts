import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-accordion-styling',
  templateUrl: './styling.html'
})
export class DemoAccordionStylingComponent {
  customClass = 'customClass';
  isFirstOpen = true;
}
