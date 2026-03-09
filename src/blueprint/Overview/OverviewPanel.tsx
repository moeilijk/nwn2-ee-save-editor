import { useState } from 'react';
import { Button, Card, Elevation, H4, InputGroup, ProgressBar, Tag, TextArea } from '@blueprintjs/core';
import { T } from '../theme';
import { CHARACTER, ABILITIES } from '../dummy-data';
import { KVRow, mod, fmtNum } from '../shared';
import { DeityDialog } from './DeityDialog';

export function OverviewPanel() {
  const hpPct = CHARACTER.hp.current / CHARACTER.hp.max;

  const [isEditingName, setIsEditingName] = useState(false);
  const [firstName, setFirstName] = useState('Khelgar');
  const [lastName, setLastName] = useState('Ironfist');
  const [isEditingBio, setIsEditingBio] = useState(false);
  const [biography, setBiography] = useState(CHARACTER.biography);
  const [isDeityOpen, setIsDeityOpen] = useState(false);
  const [deity, setDeity] = useState(CHARACTER.deity);

  return (
    <div style={{ padding: 16, display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>

      {/* ── Left: Identity + Flavor + Context ── */}
      <Card elevation={Elevation.ONE} style={{ padding: 0, background: T.surface, overflow: 'hidden' }}>

        {/* Identity */}
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
          <div style={{ display: 'flex', gap: 6, alignItems: 'center', marginBottom: 10 }}>
            {CHARACTER.classes.map((c, i) => (
              <Tag key={i} minimal round style={{ background: T.sectionBg, color: T.accent, border: `1px solid ${T.sectionBorder}`, fontSize: 12 }}>
                {c.name} {c.level}
              </Tag>
            ))}
            <span style={{ color: T.textMuted, fontSize: 12 }}>Level {CHARACTER.level}</span>
          </div>

          {/* Character details */}
          <div style={{ fontSize: 13 }}>
            <KVRow label="Race" value={CHARACTER.race} />
            <KVRow label="Gender / Age" value={`${CHARACTER.gender}, ${CHARACTER.age} yrs`} />
            <KVRow label="Alignment" value={CHARACTER.alignment} />
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

          {/* Domains */}
          {CHARACTER.domains.length > 0 && (
            <div style={{ marginTop: 8 }}>
              <KVRow label="Domains" value={CHARACTER.domains.map(d => d.name).join(', ')} />
            </div>
          )}
        </div>

        {/* Progression */}
        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontSize: 11, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.06em', color: T.accent, marginBottom: 8 }}>Progression</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px', fontSize: 13 }}>
            <KVRow label="Skill Points" value={String(CHARACTER.totalSkillPoints)} />
            <KVRow label="Total Feats" value={String(CHARACTER.totalFeats)} />
            <KVRow label="Known Spells" value={String(CHARACTER.knownSpells)} />
            <KVRow label="Size" value={CHARACTER.size} />
          </div>
        </div>

        {/* Biography */}
        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '12px 16px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
            <span style={{ fontSize: 11, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.06em', color: T.accent }}>Biography</span>
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

        {/* Campaign */}
        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px', fontSize: 13 }}>
          <div style={{ fontSize: 11, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.06em', color: T.accent, marginBottom: 8 }}>Campaign</div>
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

        {/* Quest Progress */}
        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontSize: 11, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.06em', color: T.accent, marginBottom: 8 }}>Quest Progress</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px', fontSize: 13 }}>
            <KVRow label="Completed" value={CHARACTER.campaign.questProgress.completed} />
            <KVRow label="Active" value={CHARACTER.campaign.questProgress.active} />
            <KVRow label="Completion" value={`${CHARACTER.campaign.questProgress.completionRate}%`} />
          </div>
        </div>
      </Card>

      {/* ── Right: All mechanical / numbers ── */}
      <Card elevation={Elevation.ONE} style={{ padding: 0, background: T.surface, overflow: 'hidden' }}>

        {/* Combat */}
        <div style={{ padding: '12px 16px' }}>
          <div style={{ fontSize: 11, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.06em', color: T.accent, marginBottom: 8 }}>Combat</div>
          <KVRow label="Hit Points" value={<>{CHARACTER.hp.current}<span style={{ color: T.textMuted }}>/{CHARACTER.hp.max}</span></>} />
          <ProgressBar value={hpPct} intent={hpPct > 0.5 ? 'success' : 'warning'} stripes={false} animate={false} style={{ height: 3, marginBottom: 6 }} />
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px', fontSize: 13 }}>
            <KVRow label="Armor Class" value={CHARACTER.ac} />
            <KVRow label="Speed" value={`${CHARACTER.speed} ft`} />
            <KVRow label="Base Attack" value={mod(CHARACTER.bab)} />
            <KVRow label="Initiative" value={mod(CHARACTER.initiative)} />
            <KVRow label="Melee" value={mod(CHARACTER.melee)} />
            <KVRow label="Ranged" value={mod(CHARACTER.ranged)} />
          </div>
        </div>

        {/* Saves */}
        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontSize: 11, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.06em', color: T.accent, marginBottom: 8 }}>Saves</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px', fontSize: 13 }}>
            <KVRow label="Fortitude" value={mod(CHARACTER.saves.fort)} />
            <KVRow label="Reflex" value={mod(CHARACTER.saves.ref)} />
            <KVRow label="Will" value={mod(CHARACTER.saves.will)} />
          </div>
        </div>

        {/* Ability Scores */}
        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontSize: 11, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.06em', color: T.accent, marginBottom: 8 }}>Ability Scores</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px', fontSize: 13 }}>
            {ABILITIES.map(a => (
              <KVRow key={a.name} label={a.full} value={<>{a.effective} <span style={{ color: a.modifier > 0 ? T.positive : a.modifier < 0 ? T.negative : T.textMuted }}>{mod(a.modifier)}</span></>} />
            ))}
          </div>
        </div>

        {/* Special Defenses */}
        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontSize: 11, fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.06em', color: T.accent, marginBottom: 8 }}>Special Defenses</div>
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
