import { useState } from 'react';
import { Button, Callout, Checkbox, Dialog, DialogBody, DialogFooter } from '@blueprintjs/core';
import { T } from '../theme';
import { useTranslations } from '@/hooks/useTranslations';

interface GameLaunchDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onLaunch: (closeEditor: boolean) => Promise<void>;
  saveName?: string;
  gamePathDetected?: boolean;
}

export function GameLaunchDialog({ isOpen, onClose, onLaunch, saveName, gamePathDetected = true }: GameLaunchDialogProps) {
  const t = useTranslations();
  const [closeEditor, setCloseEditor] = useState(false);
  const [isLaunching, setIsLaunching] = useState(false);

  const handleLaunch = async () => {
    setIsLaunching(true);
    try {
      await onLaunch(closeEditor);
    } finally {
      setIsLaunching(false);
    }
  };

  return (
    <Dialog
      isOpen={isOpen}
      onClose={onClose}
      title={t('gameLaunch.saveComplete')}
      className="bp-app"
      style={{ width: 440, paddingBottom: 0, background: T.surface }}
      canOutsideClickClose={!isLaunching}
      isCloseButtonShown={!isLaunching}
    >
      <DialogBody style={{ padding: 16, margin: 0, background: T.surface }}>
        <p style={{ color: T.textMuted, fontSize: 13, marginBottom: 12 }}>
          <strong style={{ color: T.text }}>{saveName || 'Character'}</strong>{' '}
          {t('gameLaunch.savedSuccessfully', { name: '' }).replace(/ $/, '')}
        </p>

        <div style={{
          background: T.sectionBg, border: `1px solid ${T.sectionBorder}`,
          borderRadius: 4, padding: 12, marginBottom: 12,
        }}>
          <div style={{ fontWeight: 500, color: T.text, fontSize: 13 }}>{t('gameLaunch.readyToVerify')}</div>
          <div style={{ fontSize: 12, color: T.textMuted, marginTop: 4 }}>{t('gameLaunch.readyToVerifyDesc')}</div>
        </div>

        {!gamePathDetected && (
          <Callout intent="warning" icon="warning-sign" style={{ marginBottom: 12, fontSize: 12 }}>
            {t('gameLaunch.gamePathWarning')}
          </Callout>
        )}

        <Checkbox
          checked={closeEditor}
          onChange={e => setCloseEditor((e.target as HTMLInputElement).checked)}
          label={t('actions.closeEditorAfterLaunch')}
          disabled={isLaunching}
          style={{ fontSize: 13, color: T.textMuted }}
        />
      </DialogBody>
      <DialogFooter
        style={{ background: T.surfaceAlt, borderTop: `1px solid ${T.borderLight}` }}
        actions={
          <>
            <Button text={t('actions.stayInEditor')} onClick={onClose} disabled={isLaunching} />
            <Button
              text={isLaunching ? t('actions.launching') : t('actions.launchGame')}
              intent="primary"
              loading={isLaunching}
              onClick={handleLaunch}
              icon="play"
            />
          </>
        }
      />
    </Dialog>
  );
}
