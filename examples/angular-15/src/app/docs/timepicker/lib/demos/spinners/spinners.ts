import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-timepicker-spinners',
  templateUrl: './spinners.html'
})
export class DemoTimepickerSpinnersComponent {
  isMeridian = false;
  showSpinners = true;
  myTime: Date = new Date();
}
