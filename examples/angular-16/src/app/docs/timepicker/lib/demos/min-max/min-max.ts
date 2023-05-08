import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-timepicker-min-max',
  templateUrl: './min-max.html'
})
export class DemoTimepickerMinMaxComponent {
  myTime: Date = new Date();
  minTime: Date = new Date();
  maxTime: Date = new Date();

  constructor() {
    this.minTime.setHours(8);
    this.minTime.setMinutes(0);
    this.maxTime.setHours(23);
    this.maxTime.setMinutes(55);
  }
}
