import { Component } from '@angular/core';
import { AbstractControl, UntypedFormControl } from '@angular/forms';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-timepicker-custom-validation',
  templateUrl: './custom-validation.html'
})
export class DemoTimepickerCustomValidationComponent {
  myTime?: Date;

  ctrl = new UntypedFormControl('', (control: AbstractControl) => {
    const value = control.value;

    if (!value) {
      return null;
    }

    const hours = value.getHours();

    if (hours < 11 || hours > 12) {
      return { outOfRange: true };
    }

    return null;
  });
}
