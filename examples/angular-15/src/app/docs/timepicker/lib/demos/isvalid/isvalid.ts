import { Component } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-timepicker-isvalid',
  templateUrl: './isvalid.html'
})
export class DemoTimepickerIsValidComponent {
  isMeridian = true;
  myTime = new Date();
  valid = true;

  isValid(event: boolean): void {
    this.valid = event;
  }
}
