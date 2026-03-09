import { HTMLTable, Tag, ProgressBar } from '@blueprintjs/core';
import { T } from '../theme';
import { GAME_STATE } from '../dummy-data';
import { SectionBar } from '../shared';

const STATUS_COLORS: Record<string, { bg: string; color: string }> = {
  active: { bg: 'rgba(90,122,90,0.15)', color: T.positive },
  completed: { bg: `${T.accent}15`, color: T.accent },
  failed: { bg: 'rgba(156,64,64,0.15)', color: T.negative },
};

export function GameStatePanel() {
  return (
    <div>
      <SectionBar title="Campaign Variables" right={<span style={{ fontSize: 11, color: T.textMuted }}>{GAME_STATE.campaignVars.length} variables</span>} />
      <div style={{ padding: '0 24px' }}>
        <HTMLTable compact striped bordered style={{ width: '100%', tableLayout: 'fixed' }}>
          <colgroup><col /><col style={{ width: 80 }} /><col style={{ width: 200 }} /></colgroup>
          <thead><tr><th>Variable</th><th style={{ textAlign: 'center' }}>Type</th><th>Value</th></tr></thead>
          <tbody>
            {GAME_STATE.campaignVars.map(v => (
              <tr key={v.name}>
                <td style={{ fontFamily: 'monospace', fontSize: 12, color: T.text }}>{v.name}</td>
                <td style={{ textAlign: 'center' }}><Tag minimal style={{ fontSize: 10, background: T.sectionBg, color: T.textMuted }}>{v.type}</Tag></td>
                <td style={{ fontFamily: 'monospace', fontSize: 12 }}>{v.value}</td>
              </tr>
            ))}
          </tbody>
        </HTMLTable>
      </div>

      <SectionBar title="Quests" right={<span style={{ fontSize: 11, color: T.textMuted }}>{GAME_STATE.quests.length} quests</span>} />
      <div style={{ padding: '0 24px' }}>
        <HTMLTable compact striped bordered style={{ width: '100%', tableLayout: 'fixed' }}>
          <colgroup><col /><col style={{ width: 100 }} /><col style={{ width: 80 }} /></colgroup>
          <thead><tr><th>Quest</th><th style={{ textAlign: 'center' }}>Status</th><th style={{ textAlign: 'center' }}>Entries</th></tr></thead>
          <tbody>
            {GAME_STATE.quests.map(q => {
              const sc = STATUS_COLORS[q.status] || { bg: T.sectionBg, color: T.textMuted };
              return (
                <tr key={q.name}>
                  <td style={{ fontWeight: 500, color: T.text }}>{q.name}</td>
                  <td style={{ textAlign: 'center' }}><Tag minimal round style={{ fontSize: 10, background: sc.bg, color: sc.color }}>{q.status}</Tag></td>
                  <td style={{ textAlign: 'center', color: T.textMuted }}>{q.entries}</td>
                </tr>
              );
            })}
          </tbody>
        </HTMLTable>
      </div>

      <SectionBar title="Companions" />
      <div style={{ padding: '0 24px' }}>
        <HTMLTable compact striped bordered style={{ width: '100%', tableLayout: 'fixed' }}>
          <colgroup><col style={{ width: 140 }} /><col /><col style={{ width: 100 }} /></colgroup>
          <thead><tr><th>Companion</th><th>Influence</th><th style={{ textAlign: 'center' }}>Status</th></tr></thead>
          <tbody>
            {GAME_STATE.companions.map(c => (
              <tr key={c.name}>
                <td style={{ fontWeight: 500, color: T.text }}>{c.name}</td>
                <td>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    <ProgressBar value={c.influence / 100}
                      intent={c.influence >= 60 ? 'success' : c.influence >= 30 ? 'none' : 'danger'}
                      stripes={false} animate={false} style={{ height: 4, flex: 1 }} />
                    <span style={{ fontSize: 12, fontWeight: 600, color: T.text, minWidth: 24 }}>{c.influence}</span>
                  </div>
                </td>
                <td style={{ textAlign: 'center' }}><Tag minimal round style={{ fontSize: 10, background: T.sectionBg, color: T.textMuted }}>{c.status}</Tag></td>
              </tr>
            ))}
          </tbody>
        </HTMLTable>
      </div>
    </div>
  );
}
