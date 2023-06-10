import { Component, OnInit } from '@angular/core';
import { UntypedFormBuilder, UntypedFormGroup } from '@angular/forms';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-buttons-checkbox-reactiveforms',
  templateUrl: './checkbox-reactiveforms.html'
})
export class DemoButtonsCheckboxReactiveFormsComponent implements OnInit {
  myForm?: UntypedFormGroup;

  constructor(private formBuilder: UntypedFormBuilder) {}

  ngOnInit() {
    this.myForm = this.formBuilder.group({
      left: false,
      middle: true,
      right: false
    });
  }
}
