import { useState } from 'react';
import { Button, Checkbox, InputGroup, Menu, MenuItem, Popover, Tab, Tabs, TextArea } from '@blueprintjs/core';
import { ParchmentDialog, StepInput } from '../shared';
import { T } from '../theme';

const DUMMY_PROPERTY_TYPES = [
  { id: 0, label: 'Enhancement Bonus' },
  { id: 1, label: 'Damage Bonus' },
  { id: 2, label: 'AC Bonus' },
  { id: 3, label: 'Ability Bonus' },
  { id: 4, label: 'Saving Throw Bonus' },
  { id: 5, label: 'Regeneration' },
  { id: 6, label: 'Keen' },
  { id: 7, label: 'Massive Criticals' },
  { id: 8, label: 'Immunity' },
  { id: 9, label: 'Skill Bonus' },
  { id: 10, label: 'Spell Resistance' },
  { id: 11, label: 'Freedom of Movement' },
];

const DUMMY_SUBTYPES: Record<number, { id: number; label: string }[]> = {
  1: [{ id: 0, label: 'Acid' }, { id: 1, label: 'Cold' }, { id: 2, label: 'Fire' }, { id: 3, label: 'Electrical' }, { id: 4, label: 'Sonic' }, { id: 5, label: 'Divine' }],
  3: [{ id: 0, label: 'Strength' }, { id: 1, label: 'Dexterity' }, { id: 2, label: 'Constitution' }, { id: 3, label: 'Intelligence' }, { id: 4, label: 'Wisdom' }, { id: 5, label: 'Charisma' }],
  4: [{ id: 0, label: 'Fortitude' }, { id: 1, label: 'Reflex' }, { id: 2, label: 'Will' }, { id: 3, label: 'Universal' }],
};

const DUMMY_VALUE_OPTIONS = Array.from({ length: 21 }, (_, i) => ({ id: i, label: `+${i}` }));

interface Property {
  typeId: number;
  subtype: number;
  value: number;
}

interface EditItemDialogProps {
  isOpen: boolean;
  onClose: () => void;
  itemName?: string;
}

function DropdownSelect({ label, value, items, onChange }: {
  label: string;
  value: string;
  items: { id: number; label: string }[];
  onChange: (id: number) => void;
}) {
  const menu = (
    <Menu style={{ maxHeight: 300, overflowY: 'auto' }}>
      {items.map(item => (
        <MenuItem key={item.id} text={item.label} active={item.label === value} onClick={() => onChange(item.id)} />
      ))}
    </Menu>
  );

  return (
    <div>
      <div style={{ fontWeight: 600, color: T.textMuted, marginBottom: 3 }}>{label}</div>
      <Popover content={menu} placement="bottom-start" minimal fill>
        <Button minimal rightIcon="caret-down" text={value} fill
          style={{ textAlign: 'left', border: `1px solid ${T.border}`, background: T.surface }}
        />
      </Popover>
    </div>
  );
}

export function EditItemDialog({ isOpen, onClose, itemName }: EditItemDialogProps) {
  const [tab, setTab] = useState<string>('basic');
  const [name, setName] = useState(itemName || 'Kamas +3');
  const [description, setDescription] = useState('A pair of finely crafted kamas imbued with elemental fire.');
  const [stackSize, setStackSize] = useState(1);
  const [charges, setCharges] = useState(0);
  const [identified, setIdentified] = useState(true);
  const [plot, setPlot] = useState(false);
  const [cursed, setCursed] = useState(false);
  const [stolen, setStolen] = useState(false);

  const [properties, setProperties] = useState<Property[]>([
    { typeId: 0, subtype: 0, value: 3 },
    { typeId: 6, subtype: 0, value: 0 },
    { typeId: 1, subtype: 2, value: 6 },
  ]);

  const handleAddProperty = () => {
    setProperties(prev => [...prev, { typeId: 0, subtype: 0, value: 0 }]);
  };

  const handleRemoveProperty = (index: number) => {
    setProperties(prev => prev.filter((_, i) => i !== index));
  };

  const handlePropertyChange = (index: number, field: keyof Property, val: number) => {
    setProperties(prev => prev.map((p, i) => {
      if (i !== index) return p;
      if (field === 'typeId') return { ...p, typeId: val, subtype: 0, value: 0 };
      return { ...p, [field]: val };
    }));
  };

  const getPropertyLabel = (id: number) => DUMMY_PROPERTY_TYPES.find(p => p.id === id)?.label || 'Unknown';
  const getSubtypeLabel = (typeId: number, subtypeId: number) => DUMMY_SUBTYPES[typeId]?.find(s => s.id === subtypeId)?.label || '-';
  const getValueLabel = (val: number) => DUMMY_VALUE_OPTIONS.find(v => v.id === val)?.label || `+${val}`;

  return (
    <ParchmentDialog
      isOpen={isOpen}
      onClose={onClose}
      onOpened={() => setTab('basic')}
      title={`Edit: ${name}`}
      width={720}
      minHeight={540}
      footerActions={
        <Button intent="primary" icon="floppy-disk" text="Save" onClick={onClose} />
      }
      footerLeft={
        <span style={{ color: T.textMuted }}>
          {properties.length} properties
        </span>
      }
    >
      <Tabs id="edit-item-tab" selectedTabId={tab} onChange={(id) => setTab(id as string)}>
        <Tab id="basic" title="Basic Info" panel={
          <div style={{ display: 'flex', flexDirection: 'column', gap: 14, paddingTop: 4 }}>
            <div>
              <label style={{ fontWeight: 600, color: T.textMuted, display: 'block', marginBottom: 4 }}>Name</label>
              <InputGroup value={name} onChange={(e) => setName(e.target.value)} fill />
            </div>

            <div>
              <label style={{ fontWeight: 600, color: T.textMuted, display: 'block', marginBottom: 4 }}>Description</label>
              <TextArea
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                fill rows={4}
                style={{ background: T.surfaceAlt, borderColor: T.border, color: T.text, resize: 'vertical' }}
              />
            </div>

            <div style={{
              display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12,
              padding: '10px 12px', background: T.sectionBg, border: `1px solid ${T.sectionBorder}`, borderRadius: 4,
            }}>
              <div>
                <label style={{ fontWeight: 600, color: T.textMuted, display: 'block', marginBottom: 4 }}>Stack Size</label>
                <StepInput value={stackSize} onValueChange={setStackSize} min={1} max={99} width={120} />
              </div>
              <div>
                <label style={{ fontWeight: 600, color: T.textMuted, display: 'block', marginBottom: 4 }}>Charges</label>
                <StepInput value={charges} onValueChange={setCharges} min={0} max={255} width={120} />
              </div>
            </div>

            <div>
              <label style={{ fontWeight: 600, color: T.textMuted, display: 'block', marginBottom: 6 }}>Flags</label>
              <div style={{
                display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 6,
                padding: '10px 12px', background: T.sectionBg, border: `1px solid ${T.sectionBorder}`, borderRadius: 4,
              }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  <Checkbox checked={identified} onChange={(e) => setIdentified((e.target as HTMLInputElement).checked)} style={{ margin: 0 }} />
                  <span style={{ color: T.text }}>Identified</span>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  <Checkbox checked={plot} onChange={(e) => setPlot((e.target as HTMLInputElement).checked)} style={{ margin: 0 }} />
                  <span style={{ color: T.accent }}>Plot Item</span>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  <Checkbox checked={cursed} onChange={(e) => setCursed((e.target as HTMLInputElement).checked)} style={{ margin: 0 }} />
                  <span style={{ color: T.negative }}>Cursed</span>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  <Checkbox checked={stolen} onChange={(e) => setStolen((e.target as HTMLInputElement).checked)} style={{ margin: 0 }} />
                  <span style={{ color: T.text }}>Stolen</span>
                </div>
              </div>
            </div>
          </div>
        } />

        <Tab id="properties" title="Properties" panel={
          <div style={{ paddingTop: 4 }}>
            <div style={{
              display: 'flex', alignItems: 'center', justifyContent: 'space-between',
              marginBottom: 10,
            }}>
              <span style={{ fontWeight: 600, color: T.text }}>Enchantments ({properties.length})</span>
              <Button small icon="add" intent="primary" text="Add Property" onClick={handleAddProperty} />
            </div>

            <div style={{ maxHeight: 400, overflowY: 'auto' }}>
              {properties.map((prop, i) => {
                const subtypes = DUMMY_SUBTYPES[prop.typeId];
                return (
                  <div key={i} style={{
                    display: 'flex', alignItems: 'center', gap: 8,
                    padding: '8px 0',
                    borderBottom: `1px solid ${T.borderLight}`,
                  }}>
                    <div style={{ flex: 1, display: 'grid', gridTemplateColumns: subtypes ? '1fr 1fr 100px' : '1fr 100px', gap: 8 }}>
                      <DropdownSelect
                        label="Property"
                        value={getPropertyLabel(prop.typeId)}
                        items={DUMMY_PROPERTY_TYPES}
                        onChange={(id) => handlePropertyChange(i, 'typeId', id)}
                      />
                      {subtypes && (
                        <DropdownSelect
                          label="Subtype"
                          value={getSubtypeLabel(prop.typeId, prop.subtype)}
                          items={subtypes}
                          onChange={(id) => handlePropertyChange(i, 'subtype', id)}
                        />
                      )}
                      <DropdownSelect
                        label="Value"
                        value={getValueLabel(prop.value)}
                        items={DUMMY_VALUE_OPTIONS}
                        onChange={(id) => handlePropertyChange(i, 'value', id)}
                      />
                    </div>
                    <Button small minimal icon="trash" intent="danger" onClick={() => handleRemoveProperty(i)} style={{ marginTop: 18 }} />
                  </div>
                );
              })}

              {properties.length === 0 && (
                <div style={{ padding: 32, textAlign: 'center', color: T.textMuted }}>
                  No properties. Click "Add Property" to add enchantments.
                </div>
              )}
            </div>
          </div>
        } />
      </Tabs>
    </ParchmentDialog>
  );
}
