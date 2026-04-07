import { useState, useEffect } from 'react';
import { Button, Dialog, DialogBody, DialogFooter, Icon, Tab, Tabs } from '@blueprintjs/core';
import { getName, getVersion } from '@tauri-apps/api/app';
import { T } from '../theme';
import { useTranslations } from '@/hooks/useTranslations';

interface AboutDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export function AboutDialog({ isOpen, onClose }: AboutDialogProps) {
  const t = useTranslations();
  const [appName, setAppName] = useState('NWN2:EE Save Editor');
  const [appVersion, setAppVersion] = useState('0.1.0');
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!isOpen) return;
    Promise.all([getName(), getVersion()])
      .then(([name, ver]) => { setAppName(name); setAppVersion(ver); })
      .catch(() => {});
  }, [isOpen]);

  const handleCopyVersion = async () => {
    await navigator.clipboard.writeText(`v${appVersion}`).catch(() => {});
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const infoPanel = (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', textAlign: 'center', gap: 16, padding: 16 }}>
      <div style={{
        width: 64, height: 64, borderRadius: 12,
        background: `linear-gradient(135deg, ${T.accent}, ${T.accentLight})`,
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        boxShadow: '0 4px 12px rgba(0,0,0,0.15)',
      }}>
        <span style={{ color: '#fff', fontWeight: 700, fontSize: 32 }}>N</span>
      </div>
      <div>
        <div style={{ fontSize: 18, fontWeight: 700, color: T.text }}>{appName}</div>
        <button
          onClick={handleCopyVersion}
          title={t('about.copyVersion')}
          style={{
            background: 'none', border: 'none', cursor: 'pointer',
            color: T.textMuted, fontSize: 13, display: 'flex', alignItems: 'center', gap: 4,
            margin: '4px auto 0',
          }}
        >
          <span style={{ fontFamily: 'monospace' }}>v{appVersion}</span>
          <Icon icon={copied ? 'tick' : 'duplicate'} size={12} color={copied ? T.positive : undefined} />
        </button>
      </div>
      <div style={{ color: T.textMuted, fontSize: 13, maxWidth: 320 }}>
        <p>{t('about.description')}</p>
        <p style={{ marginTop: 8, opacity: 0.75, fontSize: 12 }}>{t('about.builtWith')}</p>
      </div>
      <div style={{ display: 'flex', gap: 12, marginTop: 8 }}>
        <a href="https://github.com/Micromanner/nwn2-ee-save-editor" target="_blank" rel="noreferrer"
          style={{
            display: 'flex', alignItems: 'center', gap: 6, padding: '8px 16px',
            background: T.sectionBg, border: `1px solid ${T.border}`, borderRadius: 6,
            color: T.text, fontSize: 13, fontWeight: 500, textDecoration: 'none',
          }}>
          <Icon icon="git-repo" size={14} /> {t('about.github')}
        </a>
        <a href="https://github.com/Micromanner/nwn2-ee-save-editor/issues" target="_blank" rel="noreferrer"
          style={{
            display: 'flex', alignItems: 'center', gap: 6, padding: '8px 16px',
            background: T.sectionBg, border: `1px solid ${T.border}`, borderRadius: 6,
            color: T.text, fontSize: 13, fontWeight: 500, textDecoration: 'none',
          }}>
          <Icon icon="issue" size={14} /> {t('about.reportBug')}
        </a>
      </div>
    </div>
  );

  const creditsPanel = (
    <div style={{ padding: 16, fontSize: 13 }}>
      <h4 style={{ fontSize: 11, fontWeight: 700, textTransform: 'uppercase', letterSpacing: 1, color: T.textMuted, borderBottom: `1px solid ${T.border}`, paddingBottom: 4, marginBottom: 12 }}>
        {t('about.createdBy')}
      </h4>
      <p style={{ color: T.text, fontWeight: 500, marginBottom: 20 }}>Micromanner</p>

      <h4 style={{ fontSize: 11, fontWeight: 700, textTransform: 'uppercase', letterSpacing: 1, color: T.textMuted, borderBottom: `1px solid ${T.border}`, paddingBottom: 4, marginBottom: 12 }}>
        {t('about.specialThanks')}
      </h4>
      <div style={{ color: T.textMuted, lineHeight: 1.6, display: 'flex', flexDirection: 'column', gap: 8 }}>
        <p><span style={{ color: T.text, fontWeight: 500 }}>Obsidian Entertainment</span> — {t('about.obsidianCredit')}</p>
        <p><span style={{ color: T.text, fontWeight: 500 }}>Aspyr</span> — {t('about.aspyrCredit')}</p>
        <p><span style={{ color: T.text, fontWeight: 500 }}>ScripterRon</span> — {t('about.scripterronCredit')}</p>
      </div>
    </div>
  );

  const legalPanel = (
    <div style={{ padding: 16, fontSize: 13 }}>
      <h4 style={{ fontSize: 11, fontWeight: 700, textTransform: 'uppercase', letterSpacing: 1, color: T.textMuted, borderBottom: `1px solid ${T.border}`, paddingBottom: 4, marginBottom: 12 }}>
        {t('about.license')}
      </h4>
      <p style={{ color: T.textMuted, marginBottom: 20 }}>
        {t('about.licenseText')}
      </p>

      <h4 style={{ fontSize: 11, fontWeight: 700, textTransform: 'uppercase', letterSpacing: 1, color: T.textMuted, borderBottom: `1px solid ${T.border}`, paddingBottom: 4, marginBottom: 12 }}>
        {t('about.thirdParty')}
      </h4>
      <div style={{ color: T.textMuted, lineHeight: 1.6, display: 'flex', flexDirection: 'column', gap: 8 }}>
        <p>{t('about.fanContent', { appName })}</p>
        <p>{t('about.notAffiliated')}</p>
      </div>
    </div>
  );

  return (
    <Dialog
      isOpen={isOpen}
      onClose={onClose}
      title={t('about.title')}
      className="bp-app"
      style={{ width: 520, paddingBottom: 0, background: T.surface }}
      canOutsideClickClose
    >
      <DialogBody style={{ padding: 0, margin: 0, background: T.surface }}>
        <Tabs id="about-tabs" defaultSelectedTabId="info" renderActiveTabPanelOnly>
          <Tab id="info" title={t('about.appInfo')} panel={infoPanel} />
          <Tab id="credits" title={t('about.credits')} panel={creditsPanel} />
          <Tab id="legal" title={t('about.legal')} panel={legalPanel} />
        </Tabs>
      </DialogBody>
      <DialogFooter
        style={{ background: T.surfaceAlt, borderTop: `1px solid ${T.borderLight}` }}
        actions={<Button text={t('common.close')} onClick={onClose} />}
      />
    </Dialog>
  );
}
