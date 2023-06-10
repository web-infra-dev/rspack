import { Component, OnInit } from '@angular/core';
import { UntypedFormBuilder, UntypedFormGroup } from '@angular/forms';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-datepicker-reactive-forms',
  templateUrl: './reactive-forms.component.html'
})
export class DemoDatepickerReactiveFormsComponent implements OnInit {
  myForm?: UntypedFormGroup;
  constructor(private formBuilder: UntypedFormBuilder) {}

  ngOnInit() {
    this.myForm = this.formBuilder.group({
      date: null,
      range: null
    });
  }
}
