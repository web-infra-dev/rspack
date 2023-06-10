import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-timepicker-basic',
  templateUrl: './basic.html'
})
export class DemoTimepickerBasicComponent {
  mytime: Date = new Date();
}
