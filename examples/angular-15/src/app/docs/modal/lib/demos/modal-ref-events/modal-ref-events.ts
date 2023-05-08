import { ChangeDetectorRef, Component, TemplateRef } from '@angular/core';
import { BsModalRef, BsModalService } from 'ngx-bootstrap/modal';
import { combineLatest, Subscription } from 'rxjs';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-modal-ref-events',
  templateUrl: './modal-ref-events.html',
  styles: [`
    .card {
      margin-bottom: 0.75rem;
      padding: 8px;
    }
  `]
})
export class DemoModalRefEventsComponent {
  modalRef?: BsModalRef;
  subscriptions: Subscription[] = [];
  messages: string[] = [];

  constructor(private modalService: BsModalService, private changeDetection: ChangeDetectorRef) {
  }

  openModal(template: TemplateRef<any>) {
    this.messages = [];

    this.modalRef = this.modalService.show(template);
    let _combine;
    if (this.modalRef?.onHide && this.modalRef?.onHidden) {
        _combine = combineLatest(
        this.modalRef.onHide,
        this.modalRef.onHidden
      ).subscribe(() => this.changeDetection.markForCheck());
    }

      if (this.modalRef?.onHide) {
        this.subscriptions.push(
          this.modalRef.onHide.subscribe((reason: string | any) => {
            if (typeof reason !== 'string') {
              reason = `onHide(), modalId is : ${reason.id}`;
            }
            const _reason = reason ? `, dismissed by ${reason}` : '';
            this.messages.push(`onHide event has been fired${_reason}`);
          })
        );
      }

      if  (this.modalRef?.onHidden) {
        this.subscriptions.push(
          this.modalRef.onHidden.subscribe((reason: string | any) => {
            if (typeof reason !== 'string') {
              reason = `onHide(), modalId is : ${reason.id}`;
            }
            const _reason = reason ? `, dismissed by ${reason}` : '';
            this.messages.push(`onHidden event has been fired${_reason}`);
            this.unsubscribe();
          })
        );
      }

      if (_combine) {
        this.subscriptions.push(_combine);
      }
  }

  unsubscribe() {
    this.subscriptions.forEach((subscription: Subscription) => {
      subscription.unsubscribe();
    });
    this.subscriptions = [];
  }
}
