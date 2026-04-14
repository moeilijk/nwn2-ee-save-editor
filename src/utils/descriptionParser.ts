export interface DescriptionSection {
  label: string;
  text: string;
}

const SPELL_META_LABELS = [
  'Caster Level(s):',
  'Innate Level:',
  'School:',
  'Descriptor(s):',
  'Component(s):',
  'Range:',
  'Area of Effect / Target:',
  'Duration:',
  'Save:',
  'Spell Resistance:',
];

const FEAT_LABELS = [
  'Type of Feat:',
  'Prerequisite:',
  'Required For:',
  'Specifics:',
  'Normal:',
  'Use:',
  'Special:',
];

function splitBySections(text: string, labels: string[]): DescriptionSection[] {
  const pattern = labels
    .map(l => l.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'))
    .join('|');
  const regex = new RegExp(`(${pattern})`, 'g');

  const parts: DescriptionSection[] = [];
  let lastIndex = 0;
  let lastLabel = '';
  let match: RegExpExecArray | null;

  while ((match = regex.exec(text)) !== null) {
    const before = text.slice(lastIndex, match.index).trim();
    if (lastLabel && before) {
      parts.push({ label: lastLabel.replace(/:$/, ''), text: before });
    } else if (!lastLabel && before) {
      parts.push({ label: '', text: before });
    }
    lastLabel = match[1];
    lastIndex = match.index + match[0].length;
  }

  const remaining = text.slice(lastIndex).trim();
  if (lastLabel && remaining) {
    parts.push({ label: lastLabel.replace(/:$/, ''), text: remaining });
  } else if (!lastLabel && remaining) {
    parts.push({ label: '', text: remaining });
  }

  return parts;
}

export interface ParsedFeatDescription {
  sections: DescriptionSection[];
}

export function parseFeatDescription(description: string): ParsedFeatDescription {
  const sections = splitBySections(description, FEAT_LABELS);
  return { sections };
}

export interface ParsedSpellBody {
  duration: string;
  save: string;
  spellResistance: string;
  body: string;
}

const SHORT_VALUE_PATTERN = /^(Yes|No|Harmless|None|Special|See text|Will Negates|Will Half|Will Partial|Reflex Negates|Reflex Half|Reflex Partial|Fortitude Negates|Fortitude Half|Fortitude Partial)\b\s*(.*)/is;

function extractShortValue(text: string): [string, string] {
  const match = text.match(SHORT_VALUE_PATTERN);
  if (match) return [match[1], match[2]];
  return [text, ''];
}

export function parseSpellDescriptionBody(description: string): ParsedSpellBody {
  const sections = splitBySections(description, SPELL_META_LABELS);

  let duration = '';
  let save = '';
  let spellResistance = '';
  const bodyParts: string[] = [];

  for (const s of sections) {
    switch (s.label) {
      case 'Duration': duration = s.text; break;
      case 'Save': {
        const [val, rest] = extractShortValue(s.text);
        save = val;
        if (rest) bodyParts.push(rest);
        break;
      }
      case 'Spell Resistance': {
        const [val, rest] = extractShortValue(s.text);
        spellResistance = val;
        if (rest) bodyParts.push(rest);
        break;
      }
      case 'Caster Level(s)':
      case 'Innate Level':
      case 'School':
      case 'Descriptor(s)':
      case 'Component(s)':
      case 'Range':
      case 'Area of Effect / Target':
        break;
      default:
        bodyParts.push(s.text);
    }
  }

  return { duration, save, spellResistance, body: bodyParts.join(' ').trim() };
}
