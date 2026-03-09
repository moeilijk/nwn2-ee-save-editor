export const CHARACTER = {
  name: 'Khelgar Ironfist',
  race: 'Shield Dwarf',
  subrace: '',
  gender: 'Male',
  age: 82,
  alignment: 'Chaotic Good',
  alignmentLC: 25,
  alignmentGE: 85,
  deity: 'Tyr',
  background: 'Bully',
  level: 16,
  classes: [
    { name: 'Fighter', level: 4, hitDie: 10 },
    { name: 'Monk', level: 12, hitDie: 8 },
  ],
  xp: 136_000,
  xpNext: 153_000,
  gold: 48_523,
  hp: { current: 142, max: 168 },
  ac: 24,
  bab: 14,
  melee: 18,
  ranged: 16,
  initiative: 3,
  speed: 20,
  size: 'Medium',
  saves: { fort: 16, ref: 12, will: 10 },
  biography: 'A stout dwarf from Ironfist Hold who left his clan to find glory in combat. He seeks to prove himself worthy of becoming a monk of the Long Death.',
  domains: [
    { name: 'Strength' },
    { name: 'War' },
  ],
  spellResistance: 26,
  damageResistances: [
    { type: 'Fire', amount: 10 },
    { type: 'Cold', amount: 5 },
  ],
  damageImmunities: ['Poison', 'Disease'],
  totalSkillPoints: 61,
  totalFeats: 22,
  knownSpells: 12,
  campaign: {
    gameAct: 2,
    difficulty: 'Normal',
    lastSaved: 1710000000,
    campaignName: 'Neverwinter Nights 2 OC',
    moduleName: 'Act II - Crossroad Keep',
    location: 'Crossroad Keep - Courtyard',
    playTime: '48h 23m',
    language: 'English',
    questProgress: { completed: 14, active: 4, completionRate: 78 },
  },
};

export const DEITIES = [
  { name: 'Tyr', alignment: 'Lawful Good', portfolio: 'Justice, Law, War', favoredWeapon: 'Longsword', description: 'Tyr is the god of justice and law. Known as the Even-Handed, he is the leader of the Triad alongside Torm and Ilmater.' },
  { name: 'Tempus', alignment: 'Chaotic Neutral', portfolio: 'War, Battle, Warriors', favoredWeapon: 'Battleaxe', description: 'Tempus is the god of war. He is random in his favors, granting victory to one side one day and the other the next.' },
  { name: 'Mystra', alignment: 'Neutral Good', portfolio: 'Magic, Spells, The Weave', favoredWeapon: 'Shuriken', description: 'Mystra is the goddess of magic and the Weave. She provides and tends the Weave, the conduit through which mortal spellcasters channel magical energy.' },
  { name: 'Lathander', alignment: 'Neutral Good', portfolio: 'Spring, Dawn, Birth, Renewal', favoredWeapon: 'Mace', description: 'Lathander is the god of dawn, renewal, and vitality. He encourages new beginnings and the pursuit of perfection.' },
  { name: 'Kelemvor', alignment: 'Lawful Neutral', portfolio: 'Death, The Dead', favoredWeapon: 'Bastard Sword', description: 'Kelemvor is the god of death and the dead. He guides the souls of the recently deceased to the Fugue Plane.' },
  { name: 'Sune', alignment: 'Chaotic Good', portfolio: 'Beauty, Love, Passion', favoredWeapon: 'Whip', description: 'Sune is the goddess of beauty, love, and passion. She encourages the pursuit of beauty in all its forms.' },
  { name: 'Oghma', alignment: 'True Neutral', portfolio: 'Knowledge, Invention, Inspiration', favoredWeapon: 'Longsword', description: 'Oghma is the god of knowledge, invention, and inspiration. He is patron of bards and all who seek to create.' },
  { name: 'Helm', alignment: 'Lawful Neutral', portfolio: 'Guardians, Protectors, Protection', favoredWeapon: 'Bastard Sword', description: 'Helm is the god of guardians and protectors. The Great Guard is the epitome of the guardian and the ever-watchful sentry.' },
  { name: 'Ilmater', alignment: 'Lawful Good', portfolio: 'Endurance, Suffering, Martyrdom', favoredWeapon: 'Unarmed', description: 'Ilmater is the god of endurance and suffering. He offers succor and calming to those in pain and oppression.' },
  { name: 'Chauntea', alignment: 'Neutral Good', portfolio: 'Agriculture, Plants, Farmers', favoredWeapon: 'Scythe', description: 'Chauntea is the goddess of agriculture and plants. The Great Mother is beloved by all who work the soil.' },
  { name: 'Bane', alignment: 'Lawful Evil', portfolio: 'Strife, Hatred, Tyranny', favoredWeapon: 'Morningstar', description: 'Bane is the god of strife, hatred, and tyranny. The Black Lord rules through fear and seeks to control all of Faerun.' },
  { name: 'Cyric', alignment: 'Chaotic Evil', portfolio: 'Murder, Lies, Intrigue, Deception', favoredWeapon: 'Longsword', description: 'Cyric is the god of murder, lies, and deception. The Prince of Lies revels in chaos and treachery.' },
];

export const ABILITIES = [
  { name: 'STR', full: 'Strength', base: 20, effective: 22, modifier: 6, racial: 0, equip: 2, level: 2, enhance: 0 },
  { name: 'DEX', full: 'Dexterity', base: 13, effective: 16, modifier: 3, racial: 0, equip: 2, level: 0, enhance: 1 },
  { name: 'CON', full: 'Constitution', base: 16, effective: 18, modifier: 4, racial: 2, equip: 0, level: 0, enhance: 0 },
  { name: 'INT', full: 'Intelligence', base: 12, effective: 12, modifier: 1, racial: 0, equip: 0, level: 0, enhance: 0 },
  { name: 'WIS', full: 'Wisdom', base: 14, effective: 16, modifier: 3, racial: 0, equip: 2, level: 0, enhance: 0 },
  { name: 'CHA', full: 'Charisma', base: 8, effective: 8, modifier: -1, racial: -2, equip: 0, level: 0, enhance: 0 },
];

export const SAVES_DETAIL = [
  { name: 'Fortitude', total: 16, base: 10, ability: 4, equip: 2, feat: 0, racial: 0, misc: 0 },
  { name: 'Reflex', total: 12, base: 8, ability: 3, equip: 1, feat: 0, racial: 0, misc: 0 },
  { name: 'Will', total: 10, base: 5, ability: 3, equip: 2, feat: 0, racial: 0, misc: 0 },
];

export const LEVEL_HISTORY = [
  { level: 1, className: 'Fighter', hpGained: 10, featsGained: ['Power Attack', 'Cleave', 'Weapon Focus (Kama)'], skillPoints: 3, abilityIncrease: null as string | null },
  { level: 2, className: 'Fighter', hpGained: 8, featsGained: ['Weapon Specialization (Kama)'], skillPoints: 3, abilityIncrease: null },
  { level: 3, className: 'Fighter', hpGained: 9, featsGained: ['Toughness'], skillPoints: 3, abilityIncrease: null },
  { level: 4, className: 'Fighter', hpGained: 7, featsGained: [] as string[], skillPoints: 3, abilityIncrease: 'STR' },
  { level: 5, className: 'Monk', hpGained: 8, featsGained: ['Improved Unarmed Strike', 'Stunning Fist'], skillPoints: 4, abilityIncrease: null },
  { level: 6, className: 'Monk', hpGained: 6, featsGained: ['Combat Reflexes'], skillPoints: 4, abilityIncrease: null },
  { level: 7, className: 'Monk', hpGained: 7, featsGained: [] as string[], skillPoints: 4, abilityIncrease: null },
  { level: 8, className: 'Monk', hpGained: 5, featsGained: ['Ki Strike (Magic)'], skillPoints: 4, abilityIncrease: 'STR' },
  { level: 9, className: 'Monk', hpGained: 8, featsGained: ['Spring Attack'], skillPoints: 4, abilityIncrease: null },
  { level: 10, className: 'Monk', hpGained: 6, featsGained: ['Improved Evasion'], skillPoints: 4, abilityIncrease: null },
  { level: 11, className: 'Monk', hpGained: 7, featsGained: ['Diamond Body'], skillPoints: 4, abilityIncrease: null },
  { level: 12, className: 'Monk', hpGained: 8, featsGained: ['Great Cleave', 'Ki Strike (Lawful)'], skillPoints: 4, abilityIncrease: 'STR' },
  { level: 13, className: 'Monk', hpGained: 6, featsGained: ['Diamond Soul'], skillPoints: 4, abilityIncrease: null },
  { level: 14, className: 'Monk', hpGained: 7, featsGained: [] as string[], skillPoints: 4, abilityIncrease: null },
  { level: 15, className: 'Monk', hpGained: 8, featsGained: ['Improved Critical (Kama)'], skillPoints: 4, abilityIncrease: null },
  { level: 16, className: 'Monk', hpGained: 5, featsGained: ['Ki Strike (Adamantine)'], skillPoints: 4, abilityIncrease: 'STR' },
];

export const SKILLS = [
  { name: 'Concentration', total: 19, ranks: 19, abilityMod: 4, misc: 0, ability: 'CON', isClassSkill: true },
  { name: 'Craft Alchemy', total: 1, ranks: 0, abilityMod: 1, misc: 0, ability: 'INT', isClassSkill: false },
  { name: 'Diplomacy', total: -1, ranks: 0, abilityMod: -1, misc: 0, ability: 'CHA', isClassSkill: false },
  { name: 'Hide', total: 11, ranks: 8, abilityMod: 3, misc: 0, ability: 'DEX', isClassSkill: true },
  { name: 'Intimidate', total: 5, ranks: 6, abilityMod: -1, misc: 0, ability: 'CHA', isClassSkill: true },
  { name: 'Listen', total: 15, ranks: 12, abilityMod: 3, misc: 0, ability: 'WIS', isClassSkill: true },
  { name: 'Lore', total: 5, ranks: 4, abilityMod: 1, misc: 0, ability: 'INT', isClassSkill: true },
  { name: 'Move Silently', total: 14, ranks: 11, abilityMod: 3, misc: 0, ability: 'DEX', isClassSkill: true },
  { name: 'Parry', total: 6, ranks: 0, abilityMod: 6, misc: 0, ability: 'STR', isClassSkill: false },
  { name: 'Spot', total: 12, ranks: 9, abilityMod: 3, misc: 0, ability: 'WIS', isClassSkill: true },
  { name: 'Tumble', total: 22, ranks: 19, abilityMod: 3, misc: 0, ability: 'DEX', isClassSkill: true },
];

export const FEATS = {
  general: [
    { name: 'Power Attack', description: 'Trade attack bonus for damage.', icon: 'ife_powerattack' },
    { name: 'Cleave', description: 'Extra melee attack after dropping a foe.', icon: 'ife_cleave' },
    { name: 'Great Cleave', description: 'Unlimited cleave attacks per round.', icon: 'ife_greatcleave' },
    { name: 'Toughness', description: '+1 HP per level.', icon: 'ife_toughness' },
    { name: 'Combat Reflexes', description: 'Additional attacks of opportunity.', icon: 'ife_combatreflexes' },
    { name: 'Spring Attack', description: 'Move before and after melee attack.', icon: 'ife_springattack' },
    { name: 'Improved Critical (Kama)', description: 'Double critical threat range for kamas.', icon: 'ife_improvedcritical' },
  ],
  classBonusFeats: [
    { name: 'Weapon Focus (Kama)', description: '+1 attack with kamas.', icon: 'ife_weaponfocus' },
    { name: 'Weapon Specialization (Kama)', description: '+2 damage with kamas.', icon: 'ife_weaponspec' },
    { name: 'Improved Unarmed Strike', description: 'Unarmed attacks do not provoke AoO.', icon: 'ife_improvedunarmedstrike' },
    { name: 'Stunning Fist', description: 'Attempt to stun opponent with unarmed strike.', icon: 'ife_stunningfist' },
    { name: 'Improved Evasion', description: 'Half damage on failed Reflex saves.', icon: 'ife_evasion' },
    { name: 'Ki Strike (Magic)', description: 'Unarmed strikes count as magic weapons.', icon: 'ife_kistrike' },
    { name: 'Ki Strike (Lawful)', description: 'Unarmed strikes count as lawful weapons.', icon: 'ife_kistrike' },
    { name: 'Ki Strike (Adamantine)', description: 'Unarmed strikes count as adamantine.', icon: 'ife_kistrike' },
    { name: 'Diamond Body', description: 'Immunity to poison.', icon: 'ife_diamondbody' },
    { name: 'Diamond Soul', description: 'Spell resistance 10 + monk level.', icon: 'ife_diamondsoul' },
  ],
  background: [
    { name: 'Bully', description: '+2 to Intimidate checks.', icon: 'ife_bully' },
  ],
  racial: [
    { name: 'Dwarven Stonecunning', description: '+2 to Search checks in stonework areas.', icon: 'ife_stonecunning' },
    { name: 'Hardiness vs. Poisons', description: '+2 racial bonus on saves vs. poison.', icon: 'ife_hardiness' },
    { name: 'Hardiness vs. Spells', description: '+2 racial bonus on saves vs. spells.', icon: 'ife_hardiness' },
    { name: 'Darkvision', description: 'See in the dark up to 60 feet.', icon: 'ife_darkvision' },
  ],
};

export const SPELLS = {
  casterClasses: [
    { className: 'Cleric', casterLevel: 10, spellDC: 13, spellsPerDay: [4, 3, 3, 2, 1, 0, 0, 0, 0, 0] },
  ],
  known: [
    { level: 0, spells: [
      { name: 'Light', school: 'Evocation', description: 'Object shines like a torch.', memorized: 1 },
      { name: 'Cure Minor Wounds', school: 'Conjuration', description: 'Cures 1 point of damage.', memorized: 1 },
      { name: 'Resistance', school: 'Abjuration', description: '+1 on saving throws.', memorized: 1 },
      { name: 'Virtue', school: 'Transmutation', description: 'Subject gains 1 temporary hp.', memorized: 1 },
    ]},
    { level: 1, spells: [
      { name: 'Bless', school: 'Enchantment', description: 'Allies gain +1 on attack rolls and saves vs. fear.', memorized: 1 },
      { name: 'Cure Light Wounds', school: 'Conjuration', description: 'Cures 1d8+1/level damage.', memorized: 1 },
      { name: 'Shield of Faith', school: 'Abjuration', description: 'Aura grants +2 deflection bonus.', memorized: 1 },
    ]},
    { level: 2, spells: [
      { name: "Bull's Strength", school: 'Transmutation', description: 'Subject gains +4 to Str.', memorized: 1 },
      { name: 'Cure Moderate Wounds', school: 'Conjuration', description: 'Cures 2d8+1/level damage.', memorized: 1 },
      { name: 'Hold Person', school: 'Enchantment', description: 'Paralyzes one humanoid.', memorized: 1 },
    ]},
    { level: 3, spells: [
      { name: 'Cure Serious Wounds', school: 'Conjuration', description: 'Cures 3d8+1/level damage.', memorized: 1 },
      { name: 'Prayer', school: 'Enchantment', description: 'Allies +1 bonus, enemies -1 penalty.', memorized: 1 },
    ]},
    { level: 4, spells: [
      { name: 'Cure Critical Wounds', school: 'Conjuration', description: 'Cures 4d8+1/level damage.', memorized: 1 },
    ]},
  ],
};

export const INVENTORY = [
  { slot: 'Main Hand', name: 'Kamas +3', type: 'Weapon', weight: 2.0, value: 18302, properties: ['Enhancement +3', 'Keen', 'Fire Damage 1d6'] },
  { slot: 'Off Hand', name: 'Kamas +2', type: 'Weapon', weight: 2.0, value: 8302, properties: ['Enhancement +2', 'Cold Damage 1d4'] },
  { slot: 'Head', name: 'Headband of Intellect +2', type: 'Accessory', weight: 0.1, value: 4000, properties: ['Intelligence +2'] },
  { slot: 'Chest', name: "Monk's Belt of the Sun Soul", type: 'Armor', weight: 1.0, value: 25000, properties: ['AC Bonus +3', 'Wisdom +2'] },
  { slot: 'Hands', name: 'Gloves of the Long Death +3', type: 'Accessory', weight: 0.5, value: 12000, properties: ['Enhancement +3', 'Massive Criticals 1d8'] },
  { slot: 'Feet', name: 'Boots of Striding +2', type: 'Accessory', weight: 1.0, value: 5600, properties: ['Dexterity +2', 'Freedom of Movement'] },
  { slot: 'Cloak', name: 'Cloak of Fortification +2', type: 'Accessory', weight: 1.0, value: 8000, properties: ['Fortitude +2', 'Concealment 10%'] },
  { slot: 'Ring 1', name: 'Ring of Protection +2', type: 'Accessory', weight: 0.0, value: 8000, properties: ['Deflection AC +2'] },
  { slot: 'Ring 2', name: 'Ring of Regeneration', type: 'Accessory', weight: 0.0, value: 15000, properties: ['Regeneration +2'] },
  { slot: 'Belt', name: 'Belt of Storm Giant Strength', type: 'Accessory', weight: 1.0, value: 36000, properties: ['Strength +6'] },
  { slot: 'Amulet', name: 'Amulet of Natural Armor +3', type: 'Accessory', weight: 0.1, value: 18000, properties: ['Natural AC +3'] },
];

export const BACKPACK = [
  { name: 'Potion of Heal', qty: 5, weight: 0.5, value: 750 },
  { name: 'Potion of Barkskin', qty: 3, weight: 0.5, value: 300 },
  { name: 'Bolt of Lightning', qty: 12, weight: 0.1, value: 50 },
  { name: "Thieves' Tools +6", qty: 1, weight: 1.0, value: 850 },
  { name: 'Gem - Star Sapphire', qty: 2, weight: 0.0, value: 1000 },
  { name: 'Key to Crossroad Keep', qty: 1, weight: 0.0, value: 0 },
];

export const GAME_STATE = {
  campaignVars: [
    { name: 'nKhelgarMonkPath', type: 'int', value: '1' },
    { name: 'nShardraComplete', type: 'int', value: '0' },
    { name: 'nCrossroadKeepUpgrades', type: 'int', value: '5' },
    { name: 'sLastVisitedArea', type: 'string', value: 'Crossroad Keep - Courtyard' },
    { name: 'fPlayerReputation', type: 'float', value: '85.5' },
    { name: 'nActComplete', type: 'int', value: '2' },
    { name: 'nRomanceActive', type: 'int', value: '1' },
    { name: 'nShadowReaperKilled', type: 'int', value: '0' },
  ],
  quests: [
    { name: 'The Kalach-Cha', status: 'active', entries: 12 },
    { name: "Khelgar's Redemption", status: 'active', entries: 8 },
    { name: 'Crossroad Keep', status: 'active', entries: 15 },
    { name: 'The Shadow Reaper', status: 'active', entries: 3 },
    { name: 'Missing Miners', status: 'completed', entries: 6 },
    { name: 'Bandit Ambush', status: 'completed', entries: 4 },
    { name: 'The Githyanki Threat', status: 'failed', entries: 5 },
  ],
  companions: [
    { name: 'Neeshka', influence: 72, status: 'In Party' },
    { name: 'Elanee', influence: 55, status: 'In Party' },
    { name: 'Sand', influence: 48, status: 'At Keep' },
    { name: 'Casavir', influence: 60, status: 'At Keep' },
    { name: 'Grobnar', influence: 80, status: 'In Party' },
    { name: 'Qara', influence: 15, status: 'At Keep' },
    { name: 'Bishop', influence: 30, status: 'Left Party' },
  ],
};
