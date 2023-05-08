import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-timepicker-seconds',
  templateUrl: './toggle-minutes-seconds.html'
})
export class DemoTimepickerToggleMinutesSecondsComponent {
  myTime: Date = new Date();
  showMin = true;
  showSec = true;

  toggleMinutes(): void {
    this.showMin = !this.showMin;
  }

  toggleSeconds(): void {
    this.showSec = !this.showSec;
  }

}
