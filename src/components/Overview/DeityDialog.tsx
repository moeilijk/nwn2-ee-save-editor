import { useState, useMemo, useEffect } from 'react';
import { Button, InputGroup, Spinner } from '@blueprintjs/core';
import { T } from '../theme';
import { ParchmentDialog } from '../shared';
import { CharacterAPI, type Deity } from '@/services/characterApi';
import { useIcon } from '@/hooks/useIcon';

interface DeityDialogProps {
  isOpen: boolean;
  currentDeity: string;
  onClose: () => void;
  onSelect: (deityName: string) => void;
}

export function DeityDialog({ isOpen, currentDeity, onClose, onSelect }: DeityDialogProps) {
  const [search, setSearch] = useState('');
  const [selected, setSelected] = useState<string | null>(null);
  const [deities, setDeities] = useState<Deity[]>([]);
  const [loadingDeities, setLoadingDeities] = useState(false);

  useEffect(() => {
    if (!isOpen) return;
    setLoadingDeities(true);
    CharacterAPI.getAvailableDeities().then(res => {
      setDeities(res.deities);
    }).catch(() => {
      setDeities([]);
    }).finally(() => setLoadingDeities(false));
  }, [isOpen]);

  const filtered = useMemo(() => {
    if (!search) return deities;
    const q = search.toLowerCase();
    return deities.filter(d =>
      d.name.toLowerCase().includes(q) ||
      (d.portfolio ?? '').toLowerCase().includes(q)
    );
  }, [search, deities]);

  const detail = deities.find(d => d.name === selected);

  const handleOpen = () => {
    setSearch('');
    setSelected(currentDeity || null);
  };

  const handleConfirm = () => {
    if (selected) {
      onSelect(selected);
    }
    onClose();
  };

  return (
    <ParchmentDialog
      isOpen={isOpen}
      onClose={onClose}
      onOpened={handleOpen}
      title="Select Deity"
      width={720}
      footerActions={
        <Button text="Confirm" intent="primary" onClick={handleConfirm} style={{ background: T.accent }} />
      }
    >
      <div style={{ display: 'flex', gap: 0, margin: -16, height: 550, overflow: 'hidden' }}>
        <div style={{ width: 240, borderRight: `1px solid ${T.borderLight}`, display: 'flex', flexDirection: 'column', background: T.surfaceAlt }}>
          <div style={{ padding: 8 }}>
            <InputGroup
              leftIcon="search"
              placeholder="Search deities..."
              small
              value={search}
              onChange={e => setSearch(e.target.value)}
              rightElement={search ? <Button icon="cross" small minimal onClick={() => setSearch('')} /> : undefined}
            />
          </div>
          <div style={{ flex: 1, overflowY: 'auto' }}>
            {loadingDeities ? (
              <div style={{ display: 'flex', justifyContent: 'center', padding: 24 }}>
                <Spinner size={20} />
              </div>
            ) : (
              <>
                <button
                  onClick={() => setSelected(null)}
                  style={{
                    display: 'block', width: '100%', textAlign: 'left',
                    padding: '6px 12px', border: 'none', cursor: 'pointer',
                    background: selected === null ? `${T.accent}15` : 'transparent',
                    borderLeft: selected === null ? `2px solid ${T.accent}` : '2px solid transparent',
                    color: selected === null ? T.accent : T.textMuted,
                    fontSize: 13, fontWeight: selected === null ? 600 : 400,
                  }}
                >
                  None (No Deity)
                </button>
                {filtered.map(d => (
                  <button
                    key={d.name}
                    onClick={() => setSelected(d.name)}
                    style={{
                      display: 'block', width: '100%', textAlign: 'left',
                      padding: '6px 12px', border: 'none', cursor: 'pointer',
                      background: selected === d.name ? `${T.accent}15` : 'transparent',
                      borderLeft: selected === d.name ? `2px solid ${T.accent}` : '2px solid transparent',
                      color: selected === d.name ? T.accent : T.text,
                      fontSize: 13, fontWeight: selected === d.name ? 600 : 400,
                    }}
                  >
                    <div>{d.name}</div>
                    {d.alignment && <div style={{ fontSize: 11, color: T.textMuted }}>{d.alignment}</div>}
                  </button>
                ))}
                {filtered.length === 0 && !loadingDeities && (
                  <div style={{ padding: 16, textAlign: 'center', fontSize: 13, color: T.textMuted }}>No deities match your search.</div>
                )}
              </>
            )}
          </div>
        </div>

        <div style={{ flex: 1, padding: 16 }}>
          {detail ? (
            <div>
              <DeityDetailHeader deity={detail} />
              {detail.alignment && <div style={{ fontSize: 12, color: T.textMuted, marginBottom: 16 }}>{detail.alignment}</div>}

              {detail.portfolio && <DetailRow label="Portfolio" value={detail.portfolio} />}
              {detail.favored_weapon && <DetailRow label="Favored Weapon" value={detail.favored_weapon} />}

              {detail.description && (
                <div style={{ marginTop: 12, fontSize: 13, lineHeight: 1.6, color: T.textMuted }}>
                  {detail.description}
                </div>
              )}
            </div>
          ) : (
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
              <div style={{ textAlign: 'center' }}>
                <div style={{ fontSize: 14, color: T.textMuted }}>
                  {selected === null ? 'No deity selected' : 'Select a deity from the list'}
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </ParchmentDialog>
  );
}

function DeityDetailHeader({ deity }: { deity: Deity }) {
  const iconUrl = useIcon(deity.icon);
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 4 }}>
      {iconUrl && <img src={iconUrl} alt="" width={32} height={32} style={{ borderRadius: 4, flexShrink: 0 }} />}
      <div style={{ fontSize: 18, fontWeight: 700, color: T.text }}>{deity.name}</div>
    </div>
  );
}

function DetailRow({ label, value }: { label: string; value: string }) {
  return (
    <div style={{ display: 'flex', gap: 12, padding: '4px 0', fontSize: 13 }}>
      <span style={{ color: T.textMuted, width: 120, flexShrink: 0 }}>{label}</span>
      <span style={{ fontWeight: 600, color: T.text }}>{value}</span>
    </div>
  );
}
