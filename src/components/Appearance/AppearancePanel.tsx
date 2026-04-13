import { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import { Button, Card, Elevation, HTMLSelect, InputGroup, NonIdealState, Slider, Spinner, Switch } from '@blueprintjs/core';
import { GiMirrorMirror, GiMagnifyingGlass } from 'react-icons/gi';
import { GameIcon } from '../shared/GameIcon';
import { useTranslations } from '@/hooks/useTranslations';
import { useErrorHandler } from '@/hooks/useErrorHandler';
import { useCharacterContext, useSubsystem } from '@/contexts/CharacterContext';
import { CharacterStateAPI } from '@/lib/api/character-state';
import { T } from '../theme';
import { KVRow } from '../shared';
import type { AppearanceOption, AppearanceUpdates, TintChannel, TintChannels, VoiceSetInfo } from '@/lib/bindings';
import { CharacterViewer3D } from './CharacterViewer3D';
import { VariantStepper } from './VariantStepper';
import { ColorPicker } from './ColorPicker';

type PartType = 'head' | 'hair' | 'fhair' | 'wings' | 'tail' | 'helm' | 'body';

function SectionHeader({ label }: { label: string }) {
  return (
    <div style={{ fontSize: 12, fontWeight: 700, color: T.accent, marginBottom: 8, marginTop: 4 }}>
      {label}
    </div>
  );
}

export function AppearancePanel() {
  const t = useTranslations();
  const { handleError } = useErrorHandler();
  const { character } = useCharacterContext();
  const appearanceSubsystem = useSubsystem('appearance');

  const [wingOptions, setWingOptions] = useState<AppearanceOption[]>([]);
  const [tailOptions, setTailOptions] = useState<AppearanceOption[]>([]);
  const [modelRefreshKey, setModelRefreshKey] = useState(0);
  const [partRefresh, setPartRefresh] = useState<{ parts: PartType[]; key: number } | null>(null);
  const [liveHeight, setLiveHeight] = useState(0.95);
  const [liveGirth, setLiveGirth] = useState(0.95);
  const [voicesets, setVoicesets] = useState<VoiceSetInfo[]>([]);
  const [voiceFilter, setVoiceFilter] = useState('');
  const [playingResref, setPlayingResref] = useState<string | null>(null);
  const [pendingVoiceId, setPendingVoiceId] = useState<number | null>(null);
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const blobUrlRef = useRef<string | null>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const sizeDebounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  useEffect(() => {
    if (character?.id) {
      if (!appearanceSubsystem.data && !appearanceSubsystem.isLoading) {
        appearanceSubsystem.load();
      }
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [character?.id]);

  useEffect(() => {
    async function loadOptions() {
      try {
        const [wings, tails, voices] = await Promise.all([
          CharacterStateAPI.getAvailableWings(),
          CharacterStateAPI.getAvailableTails(),
          CharacterStateAPI.getAvailableVoicesets(),
        ]);
        setWingOptions(wings);
        setTailOptions(tails);
        setVoicesets(voices);
      } catch (err) {
        handleError(err);
      }
    }
    if (character?.id) {
      loadOptions();
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [character?.id]);

  const updateField = useCallback(async (updates: AppearanceUpdates, refreshParts?: PartType | PartType[]) => {
    try {
      const result = await CharacterStateAPI.updateAppearance(updates);
      appearanceSubsystem.updateData(result);
      if (refreshParts) {
        const parts = Array.isArray(refreshParts) ? refreshParts : [refreshParts];
        if (debounceRef.current) clearTimeout(debounceRef.current);
        debounceRef.current = setTimeout(() => {
          setPartRefresh({ parts, key: Date.now() });
        }, 300);
      }
    } catch (err) {
      handleError(err);
    }
  }, [appearanceSubsystem, handleError]);

  const updateTintChannel = useCallback(
    (group: 'tint_head' | 'tint_hair', channelKey: 'channel1' | 'channel2' | 'channel3') =>
      (value: TintChannel) => {
        const data = appearanceSubsystem.data;
        if (!data) return;
        const current: TintChannels = { ...data[group] };
        current[channelKey] = value;
        updateField({ [group]: current });
      },
    [appearanceSubsystem.data, updateField]
  );

  const cleanupAudio = useCallback(() => {
    if (audioRef.current) {
      audioRef.current.pause();
      audioRef.current = null;
    }
    if (blobUrlRef.current) {
      URL.revokeObjectURL(blobUrlRef.current);
      blobUrlRef.current = null;
    }
  }, []);

  const playVoicePreview = useCallback(async (resref: string) => {
    try {
      cleanupAudio();
      setPlayingResref(resref);

      const audioBytes = await CharacterStateAPI.previewVoiceset(resref);
      const blob = new Blob([new Uint8Array(audioBytes)], { type: 'audio/wav' });
      const url = URL.createObjectURL(blob);
      blobUrlRef.current = url;

      const audio = new Audio(url);
      audio.volume = 0.5;
      audioRef.current = audio;
      audio.onended = () => {
        setPlayingResref(null);
        URL.revokeObjectURL(url);
        blobUrlRef.current = null;
      };
      audio.onerror = () => {
        setPlayingResref(null);
        URL.revokeObjectURL(url);
        blobUrlRef.current = null;
      };
      await audio.play();
    } catch {
      setPlayingResref(null);
    }
  }, [cleanupAudio]);

  useEffect(() => {
    return () => { cleanupAudio(); };
  }, [cleanupAudio]);

  const groupedVoicesets = useMemo(() => {
    const filtered = voicesets.filter(v =>
      voiceFilter === '' || v.name.toLowerCase().includes(voiceFilter.toLowerCase())
    );
    const grouped = filtered.reduce((acc, v) => {
      const key = v.voice_type;
      if (!acc[key]) acc[key] = [];
      acc[key].push(v);
      return acc;
    }, {} as Record<number, VoiceSetInfo[]>);
    return Object.keys(grouped).map(Number).sort((a, b) => a - b).map(typeKey => ({
      typeKey,
      voices: grouped[typeKey],
    }));
  }, [voicesets, voiceFilter]);

  const dataHeight = appearanceSubsystem.data?.height;
  const dataGirth = appearanceSubsystem.data?.girth;
  useEffect(() => { if (dataHeight !== undefined) setLiveHeight(dataHeight); }, [dataHeight]);
  useEffect(() => { if (dataGirth !== undefined) setLiveGirth(dataGirth); }, [dataGirth]);

  if (!character) {
    return <NonIdealState icon={<GameIcon icon={GiMirrorMirror} size={40} />} title={t('character.noCharacter')} description={t('character.loadSaveFile')} />;
  }

  if (appearanceSubsystem.isLoading && !appearanceSubsystem.data) {
    return <Spinner />;
  }

  const data = appearanceSubsystem.data;
  if (!data) {
    return <Spinner />;
  }

  return (
    <div style={{ display: 'flex', height: '100%', overflow: 'hidden' }}>
      {/* Left: Controls */}
      <div style={{ width: 350, flexShrink: 0, overflow: 'auto', padding: 16, display: 'flex', flexDirection: 'column', gap: 10 }}>

        {/* Head & Hair */}
        <Card elevation={Elevation.ONE} style={{ padding: '12px 16px', background: T.surface }}>
          <SectionHeader label={t('appearance.headAndHair')} />
          <KVRow
            label={t('appearance.head')}
            value={<VariantStepper value={data.appearance_head} variants={data.available_heads} onChange={(v) => updateField({ appearance_head: v }, ['head', 'fhair'])} />}
          />
          <KVRow
            label={t('appearance.hairStyle')}
            value={<VariantStepper value={data.appearance_hair} variants={data.available_hairs} onChange={(v) => updateField({ appearance_hair: v }, 'hair')} />}
          />
          <KVRow
            label={t('appearance.facialHair')}
            value={
              <Switch
                checked={data.appearance_fhair > 0}
                disabled={!data.has_fhair_meshes}
                onChange={() => updateField({ appearance_fhair: data.appearance_fhair > 0 ? 0 : 1 }, 'fhair')}
                style={{ marginBottom: 0 }}
                title={!data.has_fhair_meshes ? t('appearance.noFacialHair') : undefined}
              />
            }
          />
        </Card>

        {/* Colors */}
        <Card elevation={Elevation.ONE} style={{ padding: '12px 16px', background: T.surface }}>
          <SectionHeader label={t('appearance.colors')} />
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            <ColorPicker label={t('appearance.skin')} value={data.tint_head.channel1} onChange={updateTintChannel('tint_head', 'channel1')} />
            <ColorPicker label={t('appearance.eyes')} value={data.tint_head.channel2} onChange={updateTintChannel('tint_head', 'channel2')} />
            <ColorPicker label={t('appearance.eyebrows')} value={data.tint_head.channel3} onChange={updateTintChannel('tint_head', 'channel3')} />
            <ColorPicker label={t('appearance.hairBase')} value={data.tint_hair.channel1} onChange={updateTintChannel('tint_hair', 'channel1')} />
            <ColorPicker label={t('appearance.hairHighlight')} value={data.tint_hair.channel2} onChange={updateTintChannel('tint_hair', 'channel2')} />
            <ColorPicker label={t('appearance.hairAccessory')} value={data.tint_hair.channel3} onChange={updateTintChannel('tint_hair', 'channel3')} />
          </div>
        </Card>

        {/* Size */}
        <Card elevation={Elevation.ONE} style={{ padding: '12px 16px', background: T.surface }}>
          <SectionHeader label={t('appearance.size')} />
          <KVRow
            label={t('appearance.height')}
            value={
              <div style={{ display: 'flex', alignItems: 'center', gap: 12, flex: 1 }}>
                <Slider
                  min={0.95} max={1.05} stepSize={0.01}
                  value={liveHeight}
                  onChange={(v) => {
                    setLiveHeight(v);
                    if (sizeDebounceRef.current) clearTimeout(sizeDebounceRef.current);
                    sizeDebounceRef.current = setTimeout(() => updateField({ height: v }), 100);
                  }}
                  labelRenderer={false}
                />
                <span style={{ fontSize: 13, fontWeight: 600, minWidth: 36, textAlign: 'right' }}>{liveHeight.toFixed(2)}</span>
              </div>
            }
          />
          <KVRow
            label={t('appearance.girth')}
            value={
              <div style={{ display: 'flex', alignItems: 'center', gap: 12, flex: 1 }}>
                <Slider
                  min={0.95} max={1.05} stepSize={0.01}
                  value={liveGirth}
                  onChange={(v) => {
                    setLiveGirth(v);
                    if (sizeDebounceRef.current) clearTimeout(sizeDebounceRef.current);
                    sizeDebounceRef.current = setTimeout(() => updateField({ girth: v }), 100);
                  }}
                  labelRenderer={false}
                />
                <span style={{ fontSize: 13, fontWeight: 600, minWidth: 36, textAlign: 'right' }}>{liveGirth.toFixed(2)}</span>
              </div>
            }
          />
        </Card>

        {/* Voice */}
        <Card elevation={Elevation.ONE} style={{ padding: '12px 16px', background: T.surface }}>
          <SectionHeader label={t('appearance.voiceSet')} />
          {(() => {
            const currentVoice = voicesets.find(v => v.id === data.soundset);
            const currentName = currentVoice?.name ?? `#${data.soundset}`;
            return (
              <div style={{ fontSize: 12, marginBottom: 8, display: 'flex', alignItems: 'center', gap: 6 }}>
                <span style={{ color: T.textMuted }}>{t('appearance.voiceCurrent')}:</span>
                <span style={{ fontWeight: 600 }}>{currentName}</span>
                {currentVoice && (
                  <Button
                    icon={playingResref === currentVoice.resref ? 'stop' : 'volume-up'}
                    minimal
                    small
                    onClick={() => {
                      if (playingResref === currentVoice.resref) {
                        if (audioRef.current) audioRef.current.pause();
                        setPlayingResref(null);
                      } else {
                        playVoicePreview(currentVoice.resref);
                      }
                    }}
                  />
                )}
              </div>
            );
          })()}
          <InputGroup
            placeholder={t('appearance.voiceSearch')}
            value={voiceFilter}
            onValueChange={setVoiceFilter}
            small
            leftIcon="search"
            style={{ marginBottom: 8 }}
          />
          <div style={{ maxHeight: 300, overflowY: 'auto', border: `1px solid ${T.border}`, borderRadius: 3 }}>
            {voicesets.length === 0 ? (
              <div style={{ padding: 12, textAlign: 'center', color: T.textMuted, fontSize: 12 }}>
                {t('appearance.voiceNone')}
              </div>
            ) : (
              (() => {
                const typeLabels: Record<number, string> = {
                  0: t('appearance.voiceTypePlayer'),
                  1: t('appearance.voiceTypeHenchman'),
                  2: t('appearance.voiceTypeNPC'),
                  3: t('appearance.voiceTypeNPC'),
                  4: t('appearance.voiceTypeCreature'),
                };

                return groupedVoicesets.map(({ typeKey, voices }) => (
                  <div key={typeKey}>
                    <div style={{
                      padding: '4px 8px',
                      fontSize: 10,
                      fontWeight: 700,
                      textTransform: 'uppercase',
                      color: T.accent,
                      background: T.surface,
                      borderBottom: `1px solid ${T.border}`,
                      position: 'sticky',
                      top: 0,
                      zIndex: 1,
                    }}>
                      {typeLabels[typeKey] ?? `Type ${typeKey}`}
                    </div>
                    {voices.map(v => {
                      const isCurrent = v.id === data.soundset;
                      const isPending = pendingVoiceId === v.id;
                      const isHighlighted = isPending || (isCurrent && pendingVoiceId === null);
                      const isPlaying = playingResref === v.resref;
                      return (
                        <div
                          key={v.id}
                          onClick={() => setPendingVoiceId(v.id === data.soundset ? null : v.id)}
                          style={{
                            display: 'flex',
                            alignItems: 'center',
                            padding: '4px 8px',
                            cursor: 'pointer',
                            background: isHighlighted ? T.accent + '22' : 'transparent',
                            borderBottom: `1px solid ${T.border}`,
                            fontSize: 12,
                            gap: 6,
                          }}
                        >
                          <span style={{
                            flex: 1,
                            fontWeight: isHighlighted ? 600 : 400,
                            color: isHighlighted ? T.accent : T.text,
                            overflow: 'hidden',
                            textOverflow: 'ellipsis',
                            whiteSpace: 'nowrap',
                          }}>
                            {v.name}
                          </span>
                          <Button
                            icon={isPlaying ? 'stop' : 'play'}
                            minimal
                            small
                            loading={isPlaying}
                            onClick={(e: React.MouseEvent) => {
                              e.stopPropagation();
                              if (isPlaying) {
                                if (audioRef.current) audioRef.current.pause();
                                setPlayingResref(null);
                              } else {
                                playVoicePreview(v.resref);
                              }
                            }}
                          />
                        </div>
                      );
                    })}
                  </div>
                ));
              })()
            )}
          </div>
          {pendingVoiceId !== null && (
            <div style={{ marginTop: 8, display: 'flex', justifyContent: 'flex-end', gap: 6 }}>
              <Button
                small
                text={t('common.cancel')}
                onClick={() => setPendingVoiceId(null)}
              />
              <Button
                small
                intent="primary"
                text={t('appearance.voiceConfirm')}
                onClick={() => {
                  updateField({ soundset: pendingVoiceId });
                  setPendingVoiceId(null);
                }}
              />
            </div>
          )}
        </Card>

        {/* Equipment Display */}
        <Card elevation={Elevation.ONE} style={{ padding: '12px 16px', background: T.surface }}>
          <SectionHeader label={t('appearance.equipment')} />
          <KVRow
            label={t('appearance.showHelmet')}
            value={
              <Switch
                checked={!data.never_draw_helmet}
                onChange={() => updateField({ never_draw_helmet: !data.never_draw_helmet }, 'helm')}
                style={{ marginBottom: 0 }}
              />
            }
          />
        </Card>

        {/* Wings & Tail */}
        <Card elevation={Elevation.ONE} style={{ padding: '12px 16px', background: T.surface }}>
          <SectionHeader label={t('appearance.wingsAndTail')} />
          <div style={{ fontSize: 13 }}>
            <KVRow
              label={t('appearance.wings')}
              value={
                <HTMLSelect
                  value={data.wings}
                  onChange={(e) => updateField({ wings: Number(e.target.value) }, 'wings')}
                  minimal
                  style={{ fontSize: 13 }}
                >
                  {wingOptions.map(opt => (
                    <option key={opt.id} value={opt.id}>{opt.name}</option>
                  ))}
                </HTMLSelect>
              }
            />
            <KVRow
              label={t('appearance.tail')}
              value={
                <HTMLSelect
                  value={data.tail}
                  onChange={(e) => updateField({ tail: Number(e.target.value) }, 'tail')}
                  minimal
                  style={{ fontSize: 13 }}
                >
                  {tailOptions.map(opt => (
                    <option key={opt.id} value={opt.id}>{opt.name}</option>
                  ))}
                </HTMLSelect>
              }
            />
          </div>
        </Card>
      </div>

      {/* Right: 3D Preview */}
      <div style={{ flex: 1, minWidth: 0 }}>
        <CharacterViewer3D
          refreshKey={modelRefreshKey}
          refreshPart={partRefresh}
          tintHead={data.tint_head}
          tintHair={data.tint_hair}
          tintCloak={data.cloak_tint}
          tintArmor={data.armor_tint}
          height={liveHeight}
          girth={liveGirth}
        />
      </div>
    </div>
  );
}
