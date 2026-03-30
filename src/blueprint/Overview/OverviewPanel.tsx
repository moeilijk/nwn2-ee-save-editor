import { useState, useEffect, useCallback } from 'react';
import { Button, Card, Elevation, H4, InputGroup, NonIdealState, ProgressBar, Spinner, TextArea } from '@blueprintjs/core';
import { useTranslations } from '@/hooks/useTranslations';
import { useErrorHandler } from '@/hooks/useErrorHandler';
import { T } from '../theme';
import { KVRow, mod, StepInput } from '../shared';
import { DeityDialog } from './DeityDialog';
import { useCharacterContext, useSubsystem } from '@/contexts/CharacterContext';
import { CharacterAPI } from '@/services/characterApi';
import { display, formatModifier, formatNumber } from '@/utils/dataHelpers';

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

const ABILITY_DEFS = [
  { key: 'Str' as const, fallbackKey: 'strength' as const, label: 'Strength' },
  { key: 'Dex' as const, fallbackKey: 'dexterity' as const, label: 'Dexterity' },
  { key: 'Con' as const, fallbackKey: 'constitution' as const, label: 'Constitution' },
  { key: 'Int' as const, fallbackKey: 'intelligence' as const, label: 'Intelligence' },
  { key: 'Wis' as const, fallbackKey: 'wisdom' as const, label: 'Wisdom' },
  { key: 'Cha' as const, fallbackKey: 'charisma' as const, label: 'Charisma' },
];

export function OverviewPanel() {
  const t = useTranslations();
  const { handleError } = useErrorHandler();
  const { character, characterId, isLoading, updateCharacterPartial } = useCharacterContext();

  const abilitySub = useSubsystem('abilityScores');
  const combatSub = useSubsystem('combat');
  const savesSub = useSubsystem('saves');

  const [isEditingName, setIsEditingName] = useState(false);
  const [firstName, setFirstName] = useState('');
  const [lastName, setLastName] = useState('');

  const [isEditingBio, setIsEditingBio] = useState(false);
  const [biography, setBiography] = useState('');
  const [savedBiography, setSavedBiography] = useState('');

  const [isDeityOpen, setIsDeityOpen] = useState(false);

  const [hp, setHp] = useState(0);
  const [maxHp, setMaxHp] = useState(1);
  const [hpSaving, setHpSaving] = useState(false);

  const [lawChaos, setLawChaos] = useState(50);
  const [goodEvil, setGoodEvil] = useState(50);
  const [alignmentSaving, setAlignmentSaving] = useState(false);

  useEffect(() => {
    if (!character) return;
    setFirstName(character.first_name ?? '');
    setLastName(character.last_name ?? '');
    const bio = character.biography ?? '';
    setBiography(bio);
    setSavedBiography(bio);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [character?.id]);

  useEffect(() => {
    if (abilitySub.data) {
      setHp(abilitySub.data.hit_points.current);
      setMaxHp(abilitySub.data.hit_points.max);
    } else if (character) {
      setHp(character.hitPoints ?? 0);
      setMaxHp(character.maxHitPoints ?? 1);
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [abilitySub.data, character?.hitPoints, character?.maxHitPoints]);

  useEffect(() => {
    if (!characterId) return;
    CharacterAPI.getAlignment(characterId).then(a => {
      setLawChaos(a.law_chaos);
      setGoodEvil(a.good_evil);
    }).catch(handleError);
  }, [characterId, handleError]);

  useEffect(() => {
    if (!characterId) return;
    abilitySub.load({ silent: true }).catch(handleError);
    combatSub.load({ silent: true }).catch(handleError);
    savesSub.load({ silent: true }).catch(handleError);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [characterId]);

  const handleNameSave = useCallback(async () => {
    if (!characterId) return;
    try {
      await CharacterAPI.updateCharacter(characterId, { first_name: firstName, last_name: lastName });
      updateCharacterPartial({ first_name: firstName, last_name: lastName, name: `${firstName} ${lastName}`.trim() });
      setIsEditingName(false);
    } catch (err) {
      handleError(err);
      setIsEditingName(false);
    }
  }, [characterId, firstName, lastName, updateCharacterPartial, handleError]);

  const handleNameCancel = useCallback(() => {
    setFirstName(character?.first_name ?? '');
    setLastName(character?.last_name ?? '');
    setIsEditingName(false);
  }, [character]);

  const handleBioSave = useCallback(async () => {
    if (!characterId) return;
    try {
      await CharacterAPI.setBiography(characterId, biography);
      updateCharacterPartial({ biography });
      setSavedBiography(biography);
    } catch (err) {
      handleError(err);
    }
    setIsEditingBio(false);
  }, [characterId, biography, updateCharacterPartial, handleError]);

  const handleBioCancel = useCallback(() => {
    setBiography(savedBiography);
    setIsEditingBio(false);
  }, [savedBiography]);

  const handleDeitySelect = useCallback(async (deityName: string) => {
    if (!characterId) return;
    try {
      await CharacterAPI.setDeity(characterId, deityName);
      updateCharacterPartial({ deity: deityName });
    } catch (err) {
      handleError(err);
    }
  }, [characterId, updateCharacterPartial, handleError]);

  const handleHpChange = useCallback(async (newCurrent: number, newMax: number) => {
    if (!characterId || hpSaving) return;
    setHpSaving(true);
    try {
      await CharacterAPI.updateHitPoints(characterId, newCurrent, newMax);
      updateCharacterPartial({ hitPoints: newCurrent, maxHitPoints: newMax });
    } catch (err) {
      handleError(err);
    }
    setHpSaving(false);
  }, [characterId, hpSaving, updateCharacterPartial, handleError]);

  const handleAlignmentSelect = useCallback(async (lc: number, ge: number) => {
    if (!characterId || alignmentSaving) return;
    setLawChaos(lc);
    setGoodEvil(ge);
    setAlignmentSaving(true);
    try {
      const res = await CharacterAPI.updateAlignment(characterId, { law_chaos: lc, good_evil: ge });
      updateCharacterPartial({ alignment: res.alignment_string });
    } catch (err) {
      handleError(err);
    }
    setAlignmentSaving(false);
  }, [characterId, alignmentSaving, updateCharacterPartial, handleError]);

  const handleAlignmentStep = useCallback(async (field: 'lc' | 'ge', value: number) => {
    const lc = field === 'lc' ? value : lawChaos;
    const ge = field === 'ge' ? value : goodEvil;
    await handleAlignmentSelect(lc, ge);
  }, [lawChaos, goodEvil, handleAlignmentSelect]);

  if (isLoading && !character) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: 400 }}>
        <Spinner size={40} />
      </div>
    );
  }

  if (!character) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: 400 }}>
        <NonIdealState icon="person" title="No character loaded" description="Load a save file to view the overview." />
      </div>
    );
  }

  const hpPct = maxHp > 0 ? hp / maxHp : 0;

  const abilityData = abilitySub.data;
  const combatData = combatSub.data;
  const savesData = savesSub.data;

  const ac = combatData?.armor_class.total ?? character.armorClass ?? 0;
  const bab = combatData?.base_attack_bonus ?? character.baseAttackBonus ?? 0;
  const melee = combatData?.attack_bonuses.melee ?? character.meleeAttackBonus ?? 0;
  const ranged = combatData?.attack_bonuses.ranged ?? character.rangedAttackBonus ?? 0;
  const initiative = combatData?.initiative.total ?? character.initiative ?? 0;

  const fort = savesData?.saves.fortitude.total ?? character.saves?.fortitude ?? 0;
  const reflex = savesData?.saves.reflex.total ?? character.saves?.reflex ?? 0;
  const will = savesData?.saves.will.total ?? character.saves?.will ?? 0;

  const classes = character.classes ?? [];
  const domains = character.domains ?? [];
  const damageResistances = character.damageResistances ?? [];
  const damageImmunities = character.damageImmunities ?? [];

  const questDetails = character.questDetails;
  const completedQuests = questDetails?.summary.completed_quests ?? character.completedQuests ?? 0;
  const activeQuests = questDetails?.summary.active_quests ?? character.currentQuests ?? 0;
  const completionRate = questDetails?.progress_stats.total_completion_rate;

  return (
    <div style={{ padding: 16, display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>

      <Card elevation={Elevation.ONE} style={{ padding: 0, background: T.surface, overflow: 'hidden' }}>

        <div style={{ padding: '14px 16px 12px' }}>
          {isEditingName ? (
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
              <InputGroup small value={firstName} onChange={e => setFirstName(e.target.value)} placeholder="First Name" style={{ width: 160 }} />
              <InputGroup small value={lastName} onChange={e => setLastName(e.target.value)} placeholder="Last Name" style={{ width: 160 }} />
              <Button icon="tick" intent="primary" small onClick={handleNameSave} />
              <Button icon="cross" minimal small onClick={handleNameCancel} />
            </div>
          ) : (
            <div style={{ display: 'flex', alignItems: 'baseline', gap: 8, marginBottom: 4 }}>
              <H4 style={{ margin: 0, color: T.text }}>{display(firstName)} {display(lastName)}</H4>
              <Button icon="edit" minimal small style={{ color: T.textMuted }} onClick={() => setIsEditingName(true)} />
            </div>
          )}
          <div style={{ color: T.textMuted, marginBottom: 10 }}>
            {classes.map(c => `${c.name} ${c.level}`).join(' / ')}
            {classes.length > 0 && <span style={{ color: T.textMuted }}>&mdash; {t('character.level')} {character.level}</span>}
          </div>

          <div style={{ fontSize: 13 }}>
            <KVRow label={t('character.race') ?? 'Race'} value={display(character.race)} />
            <KVRow label="Gender / Age" value={`${display(character.gender)}, ${display(character.age)} yrs`} />
            <KVRow label={t('character.alignment')} value={ALIGNMENT_GRID[getAlignmentIndex(lawChaos, goodEvil)]?.name ?? display(character.alignment)} />
            <KVRow label="Deity" value={
              <span style={{ display: 'inline-flex', alignItems: 'center', gap: 4 }}>
                {display(character.deity, 'None')}
                <Button icon="edit" minimal small style={{ color: T.textMuted }} onClick={() => setIsDeityOpen(true)} />
              </span>
            } />
            <KVRow label="Background" value={display(character.background?.name)} />
            <KVRow label={t('character.experience')} value={formatNumber(character.experience)} />
            <KVRow label={t('inventory.gold')} value={formatNumber(character.gold)} color={T.gold} />
          </div>

          {domains.length > 0 && (
            <div style={{ marginTop: 8 }}>
              <KVRow label="Domains" value={domains.map(d => d.name).join(', ')} />
            </div>
          )}
        </div>

        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontSize: 12, fontWeight: 700, color: T.accent, marginBottom: 8 }}>Progression</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px', fontSize: 13 }}>
            <KVRow label={t('character.skillPoints')} value={display(character.skill_points_available ?? character.totalSkillPoints)} />
            <KVRow label={t('character.totalFeats')} value={display(character.totalFeats)} />
            <KVRow label={t('character.knownSpells')} value={display(character.knownSpells)} />
            <KVRow label={t('character.size')} value={display(character.size)} />
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
                <Button small intent="primary" text={t('actions.save')} onClick={handleBioSave} />
                <Button small minimal text={t('actions.cancel')} onClick={handleBioCancel} />
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
            <KVRow label="Campaign" value={display(character.campaignName)} />
            <KVRow label="Game Act" value={character.gameAct != null ? `Act ${character.gameAct}` : '-'} />
            <KVRow label="Difficulty" value={display(character.difficultyLabel)} />
            <KVRow label="Module" value={display(character.moduleName)} />
            <KVRow label={t('character.location')} value={display(character.location)} />
            <KVRow label={t('character.playTime')} value={display(character.playTime)} />
            <KVRow label={t('character.lastSaved')} value={character.lastSavedTimestamp != null ? new Date(character.lastSavedTimestamp * 1000).toLocaleDateString() : display(character.lastSaved)} />
          </div>
        </div>

        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontSize: 12, fontWeight: 700, color: T.accent, marginBottom: 8 }}>Quest Progress</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px', fontSize: 13 }}>
            <KVRow label={t('character.questsCompleted')} value={display(completedQuests)} />
            <KVRow label={t('character.activeQuests')} value={display(activeQuests)} />
            {completionRate != null && (
              <KVRow label="Completion" value={`${Math.round(completionRate)}%`} />
            )}
          </div>
        </div>
      </Card>

      <Card elevation={Elevation.ONE} style={{ padding: 0, background: T.surface, overflow: 'hidden' }}>

        <div style={{ padding: '12px 16px' }}>
          <div style={{ fontSize: 12, fontWeight: 700, color: T.accent, marginBottom: 8 }}>{t('abilityScores.health')}</div>
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
              <StepInput value={hp} onValueChange={v => { setHp(v); handleHpChange(v, maxHp); }} min={-10} max={maxHp} width={88} />
            </div>
            <div>
              <div style={{ fontSize: 10, fontWeight: 600, color: T.textMuted, marginBottom: 3 }}>Max</div>
              <StepInput value={maxHp} onValueChange={v => { setMaxHp(v); handleHpChange(hp, v); }} min={1} max={9999} width={88} />
            </div>
          </div>
        </div>

        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontSize: 12, fontWeight: 700, color: T.accent, marginBottom: 8 }}>{t('character.combatStats')}</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px', fontSize: 13 }}>
            <KVRow label={t('character.armorClass')} value={display(ac)} />
            <KVRow label={t('character.speed')} value={character.movementSpeed != null ? `${character.movementSpeed} ft` : '-'} />
            <KVRow label={t('character.baseAttackBonus')} value={formatModifier(bab)} />
            <KVRow label={t('character.initiative')} value={formatModifier(initiative)} />
            <KVRow label={t('character.meleeAttack')} value={formatModifier(melee)} />
            <KVRow label={t('character.rangedAttack')} value={formatModifier(ranged)} />
          </div>
        </div>

        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontSize: 12, fontWeight: 700, color: T.accent, marginBottom: 8 }}>{t('abilityScores.savingThrows')}</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px', fontSize: 13 }}>
            <KVRow label={t('abilityScores.fortitude')} value={formatModifier(fort)} />
            <KVRow label={t('abilityScores.reflex')} value={formatModifier(reflex)} />
            <KVRow label={t('abilityScores.will')} value={formatModifier(will)} />
          </div>
        </div>

        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontSize: 12, fontWeight: 700, color: T.accent, marginBottom: 8 }}>{t('abilityScores.abilityScores')}</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px', fontSize: 13 }}>
            {ABILITY_DEFS.map(a => {
              const score = abilityData?.effective_scores[a.key] ?? character.abilities?.[a.fallbackKey];
              const modifier = abilityData?.modifiers[a.key];
              const modVal = modifier != null ? modifier : score != null ? Math.floor((score - 10) / 2) : null;
              return (
                <KVRow key={a.key} label={a.label} value={
                  <>
                    {display(score)}
                    {modVal != null && (
                      <span style={{ marginLeft: 4, color: modVal > 0 ? T.positive : modVal < 0 ? T.negative : T.textMuted }}>
                        {mod(modVal)}
                      </span>
                    )}
                  </>
                } />
              );
            })}
          </div>
        </div>

        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontSize: 12, fontWeight: 700, color: T.accent, marginBottom: 8 }}>{t('character.alignment')}</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 3, marginBottom: 10 }}>
            {ALIGNMENT_GRID.map((a, i) => {
              const active = getAlignmentIndex(lawChaos, goodEvil) === i;
              return (
                <button
                  key={a.name}
                  onClick={() => handleAlignmentSelect(a.lc, a.ge)}
                  disabled={alignmentSaving}
                  style={{
                    padding: '6px 4px', fontSize: 11, fontWeight: active ? 700 : 500,
                    lineHeight: 1.2, textAlign: 'center',
                    border: `2px solid ${active ? a.color : T.borderLight}`,
                    borderRadius: 4,
                    background: active ? `${a.color}20` : T.surface,
                    color: active ? a.color : T.textMuted,
                    cursor: alignmentSaving ? 'not-allowed' : 'pointer', transition: 'all 0.15s',
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
              <StepInput value={lawChaos} onValueChange={v => handleAlignmentStep('lc', v)} min={0} max={100} width={88} />
            </div>
            <div>
              <div style={{ fontSize: 10, fontWeight: 600, color: T.textMuted, marginBottom: 3 }}>Good - Evil</div>
              <StepInput value={goodEvil} onValueChange={v => handleAlignmentStep('ge', v)} min={0} max={100} width={88} />
            </div>
          </div>
        </div>

        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontSize: 12, fontWeight: 700, color: T.accent, marginBottom: 8 }}>Special Defenses</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px', fontSize: 13 }}>
            <KVRow label={t('character.spellResistance')} value={display(character.spellResistance)} />
            {damageImmunities.length > 0 && (
              <KVRow label={t('character.immunities')} value={damageImmunities.join(', ')} />
            )}
            {damageResistances.map(r => (
              <KVRow key={r.type} label={r.type} value={r.amount} />
            ))}
          </div>
        </div>
      </Card>

      <DeityDialog
        isOpen={isDeityOpen}
        currentDeity={character.deity ?? ''}
        onClose={() => setIsDeityOpen(false)}
        onSelect={handleDeitySelect}
      />
    </div>
  );
}
