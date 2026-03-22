import { Button, Dialog, DialogBody, DialogFooter } from '@blueprintjs/core';
import type { ReactNode } from 'react';
import { T } from '../theme';

interface ParchmentDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onOpened?: () => void;
  title: string;
  width?: number;
  children: ReactNode;
  footerActions: ReactNode;
  footerLeft?: ReactNode;
}

export function ParchmentDialog({
  isOpen, onClose, onOpened, title, width = 480,
  children, footerActions, footerLeft,
}: ParchmentDialogProps) {
  return (
    <Dialog
      isOpen={isOpen}
      onClose={onClose}
      onOpened={onOpened}
      title={title}
      style={{ width, paddingBottom: 0, background: T.surface }}
      canOutsideClickClose
    >
      <div style={{ background: T.surface }}>
        <DialogBody style={{ padding: 16, margin: 0, background: T.surface }}>
          {children}
        </DialogBody>
        <DialogFooter
          style={{ background: T.surfaceAlt, borderTop: `1px solid ${T.borderLight}` }}
          actions={
            <>
              <Button text="Cancel" onClick={onClose} />
              {footerActions}
            </>
          }
        >
          {footerLeft}
        </DialogFooter>
      </div>
    </Dialog>
  );
}
