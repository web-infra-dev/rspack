import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-timepicker-placeholder',
  templateUrl: './placeholder.html'
})
export class DemoTimepickerPlaceholderComponent {
  hoursPlaceholder = 'hh';
  minutesPlaceholder = 'mm';
  secondsPlaceholder = 'ss';
}

