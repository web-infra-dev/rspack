import { ChangeDetectionStrategy, Component, EventEmitter, Input, OnDestroy, OnInit, Output } from '@angular/core';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'sub-component',
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './sub-component.html',
  styleUrls: ['./sub-component.scss']
})
export class SubComponent implements OnInit, OnDestroy {
  @Input()
  name?: string;

  // eslint-disable-next-line @angular-eslint/no-output-on-prefix
  @Output()
    // eslint-disable-next-line @angular-eslint/no-output-on-prefix
  onInit = new EventEmitter<void>();

  // eslint-disable-next-line @angular-eslint/no-output-on-prefix
  @Output()
    // eslint-disable-next-line @angular-eslint/no-output-on-prefix
  onDestroy = new EventEmitter<void>();

  ngOnInit() {
    Promise.resolve().then(() =>
      this.onInit.emit()
    );
  }

  ngOnDestroy() {
    this.onDestroy.emit();
  }
}
