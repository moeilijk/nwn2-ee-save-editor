import { Button, Dialog, DialogBody, DialogFooter } from '@blueprintjs/core';
import type { ReactNode } from 'react';
import { T } from '../theme';

interface ParchmentDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onOpened?: () => void;
  title: string;
  width?: number;
  minHeight?: number;
  children: ReactNode;
  footerActions: ReactNode;
  footerLeft?: ReactNode;
}

export function ParchmentDialog({
  isOpen, onClose, onOpened, title, width = 480, minHeight,
  children, footerActions, footerLeft,
}: ParchmentDialogProps) {
  return (
    <Dialog
      isOpen={isOpen}
      onClose={onClose}
      onOpened={onOpened}
      title={title}
      className="bp-app"
      style={{ width, minHeight, paddingBottom: 0, background: T.surface }}
      canOutsideClickClose
    >
      <div style={{ background: T.surface, display: 'flex', flexDirection: 'column', flex: 1, minHeight: 0 }}>
        <DialogBody style={{ padding: 16, margin: 0, background: T.surface, flex: 1 }}>
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
