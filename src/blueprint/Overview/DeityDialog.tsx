import { useState, useMemo } from 'react';
import { Button, Dialog, DialogBody, DialogFooter, InputGroup } from '@blueprintjs/core';
import { T } from '../theme';
import { DEITIES } from '../dummy-data';

interface DeityDialogProps {
  isOpen: boolean;
  currentDeity: string;
  onClose: () => void;
  onSelect: (deityName: string) => void;
}

export function DeityDialog({ isOpen, currentDeity, onClose, onSelect }: DeityDialogProps) {
  const [search, setSearch] = useState('');
  const [selected, setSelected] = useState<string | null>(null);

  const filtered = useMemo(() => {
    if (!search) return DEITIES;
    const q = search.toLowerCase();
    return DEITIES.filter(d =>
      d.name.toLowerCase().includes(q) ||
      d.portfolio.toLowerCase().includes(q)
    );
  }, [search]);

  const detail = DEITIES.find(d => d.name === selected);

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
    <Dialog
      isOpen={isOpen}
      onClose={onClose}
      onOpened={handleOpen}
      title="Select Deity"
      style={{ width: 720, paddingBottom: 0, background: T.surface }}
      canOutsideClickClose
    >
      <DialogBody style={{ display: 'flex', gap: 0, padding: 0, margin: 0, minHeight: 400 }}>
        {/* Left: search + list */}
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
                <div style={{ fontSize: 11, color: T.textMuted }}>{d.alignment}</div>
              </button>
            ))}
            {filtered.length === 0 && (
              <div style={{ padding: 16, textAlign: 'center', fontSize: 13, color: T.textMuted }}>No deities match your search.</div>
            )}
          </div>
        </div>

        {/* Right: detail */}
        <div style={{ flex: 1, overflowY: 'auto', padding: 16 }}>
          {detail ? (
            <div>
              <div style={{ fontSize: 18, fontWeight: 700, color: T.text, marginBottom: 4 }}>{detail.name}</div>
              <div style={{ fontSize: 12, color: T.textMuted, marginBottom: 16 }}>{detail.alignment}</div>

              <DetailRow label="Portfolio" value={detail.portfolio} />
              <DetailRow label="Favored Weapon" value={detail.favoredWeapon} />

              <div style={{ marginTop: 12, fontSize: 13, lineHeight: 1.6, color: T.textMuted }}>
                {detail.description}
              </div>
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
      </DialogBody>

      <DialogFooter
        actions={
          <>
            <Button text="Cancel" onClick={onClose} />
            <Button text="Confirm" intent="primary" onClick={handleConfirm} style={{ background: T.accent }} />
          </>
        }
      />
    </Dialog>
  );
}

function DetailRow({ label, value }: { label: string; value: string }) {
  return (
    <div style={{ display: 'flex', justifyContent: 'space-between', padding: '4px 0', fontSize: 13 }}>
      <span style={{ color: T.textMuted }}>{label}</span>
      <span style={{ fontWeight: 600, color: T.text }}>{value}</span>
    </div>
  );
}
