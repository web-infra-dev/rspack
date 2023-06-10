import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-timepicker-custom-meridian',
  templateUrl: './custom-meridian.html'
})
export class DemoTimepickerCustomMeridianComponent {
  mytime: Date = new Date();
  meridians = ['AM(Midnight to Noon)', 'PM(Noon to Midnight)'];
}
