import { useState } from 'react';
import { Button, Card, Elevation, H4, InputGroup, ProgressBar, Tag, TextArea } from '@blueprintjs/core';
import { T } from '../theme';
import { CHARACTER, ABILITIES, VITAL_STATS } from '../dummy-data';
import { KVRow, mod, fmtNum, StepInput } from '../shared';
import { DeityDialog } from './DeityDialog';

const ALIGNMENT_GRID = [
  { name: 'Lawful Good', lc: 85, ge: 85, color: '#FFD700' },
  { name: 'Neutral Good', lc: 50, ge: 85, color: '#87CEEB' },
  { name: 'Chaotic Good', lc: 15, ge: 85, color: '#228B22' },
  { name: 'Lawful Neutral', lc: 85, ge: 50, color: '#71797E' },
  { name: 'True Neutral', lc: 50, ge: 50, color: '#A0522D' },
  { name: 'Chaotic Neutral', lc: 15, ge: 50, color: '#FF4500' },
  { name: 'Lawful Evil', lc: 85, ge: 15, color: '#8B0000' },
  { name: 'Neutral Evil', lc: 50, ge: 15, color: '#556B2F' },
  { name: 'Chaotic Evil', lc: 15, ge: 15, color: '#483D8B' },
];

function getAlignmentIndex(lc: number, ge: number): number {
  const col = lc >= 70 ? 0 : lc <= 30 ? 2 : 1;
  const row = ge >= 70 ? 0 : ge <= 30 ? 2 : 1;
  return row * 3 + col;
}

export function OverviewPanel() {
  const [isEditingName, setIsEditingName] = useState(false);
  const [firstName, setFirstName] = useState('Khelgar');
  const [lastName, setLastName] = useState('Ironfist');
  const [isEditingBio, setIsEditingBio] = useState(false);
  const [biography, setBiography] = useState(CHARACTER.biography);
  const [isDeityOpen, setIsDeityOpen] = useState(false);
  const [deity, setDeity] = useState(CHARACTER.deity);

  const [hp, setHp] = useState(VITAL_STATS.hitPoints);
  const [maxHp, setMaxHp] = useState(VITAL_STATS.maxHitPoints);
  const hpPct = maxHp > 0 ? hp / maxHp : 0;

  const [lawChaos, setLawChaos] = useState(25);
  const [goodEvil, setGoodEvil] = useState(85);

  return (
    <div style={{ padding: 16, display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>

      <Card elevation={Elevation.ONE} style={{ padding: 0, background: T.surface, overflow: 'hidden' }}>

        <div style={{ padding: '14px 16px 12px' }}>
          {isEditingName ? (
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
              <InputGroup small value={firstName} onChange={e => setFirstName(e.target.value)} placeholder="First Name" style={{ width: 160 }} />
              <InputGroup small value={lastName} onChange={e => setLastName(e.target.value)} placeholder="Last Name" style={{ width: 160 }} />
              <Button icon="tick" intent="primary" small onClick={() => setIsEditingName(false)} />
              <Button icon="cross" minimal small onClick={() => { setFirstName('Khelgar'); setLastName('Ironfist'); setIsEditingName(false); }} />
            </div>
          ) : (
            <div style={{ display: 'flex', alignItems: 'baseline', gap: 8, marginBottom: 4 }}>
              <H4 style={{ margin: 0, color: T.text }}>{firstName} {lastName}</H4>
              <Button icon="edit" minimal small style={{ color: T.textMuted }} onClick={() => setIsEditingName(true)} />
            </div>
          )}
          <div style={{ color: T.textMuted, marginBottom: 10 }}>
            {CHARACTER.classes.map(c => `${c.name} ${c.level}`).join(' / ')} <span style={{ color: T.textMuted }}>&mdash; Level {CHARACTER.level}</span>
          </div>

          <div style={{ fontSize: 13 }}>
            <KVRow label="Race" value={CHARACTER.race} />
            <KVRow label="Gender / Age" value={`${CHARACTER.gender}, ${CHARACTER.age} yrs`} />
            <KVRow label="Alignment" value={ALIGNMENT_GRID[getAlignmentIndex(lawChaos, goodEvil)]?.name ?? CHARACTER.alignment} />
            <KVRow label="Deity" value={
              <span style={{ display: 'inline-flex', alignItems: 'center', gap: 4 }}>
                {deity || 'None'}
                <Button icon="edit" minimal small style={{ color: T.textMuted }} onClick={() => setIsDeityOpen(true)} />
              </span>
            } />
            <KVRow label="Background" value={CHARACTER.background} />
            <KVRow label="Experience" value={fmtNum(CHARACTER.xp)} />
            <KVRow label="Gold" value={fmtNum(CHARACTER.gold)} color={T.gold} />
          </div>

          {CHARACTER.domains.length > 0 && (
            <div style={{ marginTop: 8 }}>
              <KVRow label="Domains" value={CHARACTER.domains.map(d => d.name).join(', ')} />
            </div>
          )}
        </div>

        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontSize: 12, fontWeight: 700, color: T.accent, marginBottom: 8 }}>Progression</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px', fontSize: 13 }}>
            <KVRow label="Skill Points" value={String(CHARACTER.totalSkillPoints)} />
            <KVRow label="Total Feats" value={String(CHARACTER.totalFeats)} />
            <KVRow label="Known Spells" value={String(CHARACTER.knownSpells)} />
            <KVRow label="Size" value={CHARACTER.size} />
          </div>
        </div>

        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '12px 16px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
            <span style={{ fontSize: 12, fontWeight: 700, color: T.accent }}>Biography</span>
            {!isEditingBio && (
              <Button icon="edit" minimal small style={{ color: T.textMuted }} onClick={() => setIsEditingBio(true)} />
            )}
          </div>
          {isEditingBio ? (
            <div>
              <TextArea fill value={biography} onChange={e => setBiography(e.target.value)} rows={4}
                style={{ background: T.surface, borderColor: T.border, color: T.text, fontSize: 13, resize: 'vertical' }} />
              <div style={{ display: 'flex', gap: 8, marginTop: 8 }}>
                <Button small intent="primary" text="Save" onClick={() => setIsEditingBio(false)} />
                <Button small minimal text="Cancel" onClick={() => { setBiography(CHARACTER.biography); setIsEditingBio(false); }} />
              </div>
            </div>
          ) : (
            <p style={{ margin: 0, fontSize: 13, lineHeight: 1.6, color: T.textMuted }}>
              {biography || 'No biography written'}
            </p>
          )}
        </div>

        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px', fontSize: 13 }}>
          <div style={{ fontSize: 12, fontWeight: 700, color: T.accent, marginBottom: 8 }}>Campaign</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px' }}>
            <KVRow label="Campaign" value={CHARACTER.campaign.campaignName} />
            <KVRow label="Game Act" value={`Act ${CHARACTER.campaign.gameAct}`} />
            <KVRow label="Difficulty" value={CHARACTER.campaign.difficulty} />
            <KVRow label="Module" value={CHARACTER.campaign.moduleName} />
            <KVRow label="Location" value={CHARACTER.campaign.location} />
            <KVRow label="Play Time" value={CHARACTER.campaign.playTime} />
            <KVRow label="Last Saved" value={new Date(CHARACTER.campaign.lastSaved * 1000).toLocaleDateString()} />
          </div>
        </div>

        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontSize: 12, fontWeight: 700, color: T.accent, marginBottom: 8 }}>Quest Progress</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px', fontSize: 13 }}>
            <KVRow label="Completed" value={CHARACTER.campaign.questProgress.completed} />
            <KVRow label="Active" value={CHARACTER.campaign.questProgress.active} />
            <KVRow label="Completion" value={`${CHARACTER.campaign.questProgress.completionRate}%`} />
          </div>
        </div>
      </Card>

      <Card elevation={Elevation.ONE} style={{ padding: 0, background: T.surface, overflow: 'hidden' }}>

        <div style={{ padding: '12px 16px' }}>
          <div style={{ fontSize: 12, fontWeight: 700, color: T.accent, marginBottom: 8 }}>Health</div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 8 }}>
            <div style={{ flex: 1 }}>
              <ProgressBar value={hpPct} intent={hpPct >= 0.7 ? 'success' : hpPct >= 0.3 ? 'warning' : 'danger'} stripes={false} animate={false} style={{ height: 4 }} />
            </div>
            <span style={{ fontSize: 13, fontWeight: 700, color: T.text, whiteSpace: 'nowrap' }}>
              {hp} / {maxHp}
              <span style={{ fontSize: 11, fontWeight: 400, color: T.textMuted, marginLeft: 4 }}>({Math.round(hpPct * 100)}%)</span>
            </span>
          </div>
          <div style={{ display: 'flex', gap: 10 }}>
            <div>
              <div style={{ fontSize: 10, fontWeight: 600, color: T.textMuted, marginBottom: 3 }}>Current</div>
              <StepInput value={hp} onValueChange={v => setHp(v)} min={-10} max={maxHp} width={88} />
            </div>
            <div>
              <div style={{ fontSize: 10, fontWeight: 600, color: T.textMuted, marginBottom: 3 }}>Max</div>
              <StepInput value={maxHp} onValueChange={v => setMaxHp(v)} min={1} max={999} width={88} />
            </div>
          </div>
        </div>

        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontSize: 12, fontWeight: 700, color: T.accent, marginBottom: 8 }}>Combat</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px', fontSize: 13 }}>
            <KVRow label="Armor Class" value={CHARACTER.ac} />
            <KVRow label="Speed" value={`${CHARACTER.speed} ft`} />
            <KVRow label="Base Attack" value={mod(CHARACTER.bab)} />
            <KVRow label="Initiative" value={mod(CHARACTER.initiative)} />
            <KVRow label="Melee" value={mod(CHARACTER.melee)} />
            <KVRow label="Ranged" value={mod(CHARACTER.ranged)} />
          </div>
        </div>

        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontSize: 12, fontWeight: 700, color: T.accent, marginBottom: 8 }}>Saves</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px', fontSize: 13 }}>
            <KVRow label="Fortitude" value={mod(CHARACTER.saves.fort)} />
            <KVRow label="Reflex" value={mod(CHARACTER.saves.ref)} />
            <KVRow label="Will" value={mod(CHARACTER.saves.will)} />
          </div>
        </div>

        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontSize: 12, fontWeight: 700, color: T.accent, marginBottom: 8 }}>Ability Scores</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px', fontSize: 13 }}>
            {ABILITIES.map(a => (
              <KVRow key={a.name} label={a.full} value={<>{a.effective} <span style={{ color: a.modifier > 0 ? T.positive : a.modifier < 0 ? T.negative : T.textMuted }}>{mod(a.modifier)}</span></>} />
            ))}
          </div>
        </div>

        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontSize: 12, fontWeight: 700, color: T.accent, marginBottom: 8 }}>Alignment</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 3, marginBottom: 10 }}>
            {ALIGNMENT_GRID.map((a, i) => {
              const active = getAlignmentIndex(lawChaos, goodEvil) === i;
              return (
                <button
                  key={a.name}
                  onClick={() => { setLawChaos(a.lc); setGoodEvil(a.ge); }}
                  style={{
                    padding: '6px 4px', fontSize: 11, fontWeight: active ? 700 : 500,
                    lineHeight: 1.2, textAlign: 'center',
                    border: `2px solid ${active ? a.color : T.borderLight}`,
                    borderRadius: 4,
                    background: active ? `${a.color}20` : T.surface,
                    color: active ? a.color : T.textMuted,
                    cursor: 'pointer', transition: 'all 0.15s',
                  }}
                >
                  {a.name}
                </button>
              );
            })}
          </div>
          <div style={{ display: 'flex', gap: 10 }}>
            <div>
              <div style={{ fontSize: 10, fontWeight: 600, color: T.textMuted, marginBottom: 3 }}>Law - Chaos</div>
              <StepInput value={lawChaos} onValueChange={v => setLawChaos(v)} min={0} max={100} width={88} />
            </div>
            <div>
              <div style={{ fontSize: 10, fontWeight: 600, color: T.textMuted, marginBottom: 3 }}>Good - Evil</div>
              <StepInput value={goodEvil} onValueChange={v => setGoodEvil(v)} min={0} max={100} width={88} />
            </div>
          </div>
        </div>

        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontSize: 12, fontWeight: 700, color: T.accent, marginBottom: 8 }}>Special Defenses</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px', fontSize: 13 }}>
            <KVRow label="Spell Resistance" value={CHARACTER.spellResistance} />
            <KVRow label="Immunities" value={CHARACTER.damageImmunities.join(', ')} />
            {CHARACTER.damageResistances.map(r => (
              <KVRow key={r.type} label={r.type} value={r.amount} />
            ))}
          </div>
        </div>
      </Card>

      <DeityDialog
        isOpen={isDeityOpen}
        currentDeity={deity}
        onClose={() => setIsDeityOpen(false)}
        onSelect={setDeity}
      />
    </div>
  );
}
