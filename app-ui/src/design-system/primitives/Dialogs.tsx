import { AlertDialog, Dialog } from "radix-ui";
import type { ReactNode } from "react";

interface ModalProps {
  children: ReactNode;
  description?: string;
  onOpenChange: (open: boolean) => void;
  open: boolean;
  title: string;
}

export function Modal({ children, description, onOpenChange, open, title }: ModalProps) {
  return (
    <Dialog.Root onOpenChange={onOpenChange} open={open}>
      <Dialog.Portal>
        <Dialog.Overlay className="pb-dialog-overlay" />
        <Dialog.Content className="pb-dialog-content">
          <Dialog.Title>{title}</Dialog.Title>
          {description ? <Dialog.Description className="pb-muted">{description}</Dialog.Description> : null}
          {children}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

interface ConfirmDialogProps {
  cancelLabel: string;
  confirmLabel: string;
  description: string;
  onConfirm: () => void;
  onOpenChange: (open: boolean) => void;
  open: boolean;
  title: string;
}

export function ConfirmDialog(props: ConfirmDialogProps) {
  return (
    <AlertDialog.Root onOpenChange={props.onOpenChange} open={props.open}>
      <AlertDialog.Portal>
        <AlertDialog.Overlay className="pb-dialog-overlay" />
        <AlertDialog.Content className="pb-dialog-content">
          <AlertDialog.Title>{props.title}</AlertDialog.Title>
          <AlertDialog.Description className="pb-muted">{props.description}</AlertDialog.Description>
          <div className="pb-dialog-actions">
            <AlertDialog.Cancel className="pb-button">{props.cancelLabel}</AlertDialog.Cancel>
            <AlertDialog.Action className="pb-button pb-button--danger" onClick={props.onConfirm}>{props.confirmLabel}</AlertDialog.Action>
          </div>
        </AlertDialog.Content>
      </AlertDialog.Portal>
    </AlertDialog.Root>
  );
}
