import { ChangeDetectorRef, Component, TemplateRef } from '@angular/core';
import { BsModalRef, BsModalService } from 'ngx-bootstrap/modal';
import { combineLatest, Subscription } from 'rxjs';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-modal-service-events',
  templateUrl: './service-events.html',
  styles: [`
    .card {
      margin-bottom: 0.75rem;
      padding: 8px;
    }
  `]
})
export class DemoModalServiceEventsComponent {
  modalRef?: BsModalRef;
  subscriptions: Subscription[] = [];
  messages: string[] = [];

  constructor(private modalService: BsModalService, private changeDetection: ChangeDetectorRef) {
  }

  openModal(template: TemplateRef<any>) {
    this.messages = [];

    const _combine = combineLatest(
      this.modalService.onShow,
      this.modalService.onShown,
      this.modalService.onHide,
      this.modalService.onHidden
    ).subscribe(() => this.changeDetection.markForCheck());

    this.subscriptions.push(
      this.modalService.onShow.subscribe(() => {
        this.messages.push(`onShow event has been fired`);
      })
    );
    this.subscriptions.push(
      this.modalService.onShown.subscribe(() => {
        this.messages.push(`onShown event has been fired`);
      })
    );
    this.subscriptions.push(
      this.modalService.onHide.subscribe((reason: string | any) => {
        if (typeof reason !== 'string') {
          reason = `onHide(), modalId is : ${reason.id}`;
        }
        const _reason = reason ? `, dismissed by ${reason}` : '';
        this.messages.push(`onHide event has been fired${_reason}`);
      })
    );
    this.subscriptions.push(
      this.modalService.onHidden.subscribe((reason: string | any) => {
        if (typeof reason !== 'string') {
          reason = `onHide(), modalId is : ${reason.id}`;
        }
        const _reason = reason ? `, dismissed by ${reason}` : '';
        this.messages.push(`onHidden event has been fired${_reason}`);
        this.unsubscribe();
      })
    );

    this.subscriptions.push(_combine);

    this.modalRef = this.modalService.show(template);
  }

  unsubscribe() {
    this.subscriptions.forEach((subscription: Subscription) => {
      subscription.unsubscribe();
    });
    this.subscriptions = [];
  }
}
