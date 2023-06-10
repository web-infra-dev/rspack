import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-accordion-one-time',
  templateUrl: './one-at-a-time.html'
})
export class DemoAccordionOneAtATimeComponent {
  oneAtATime = true;
}
