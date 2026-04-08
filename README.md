# NWN2 Enhanced Edition Save Editor

A desktop application for editing Neverwinter Nights 2 Enhanced Edition saves.

Built with Tauri, Rust, and Vite. Fully offline, available as a portable `.exe` or installer.

## Features

### Character Editing
- **Ability Scores** - Edit base scores with point-buy tracking, view racial modifiers, level bonuses, and equipment bonuses
- **Classes & Levels** - Multiclass management, level history tracking and experience points
- **Feats** - Add/remove feats
- **Skills** - Edit skills with rank allocation, class skill detection, and skill point budgets
- **Spells** - Manage known/memorized spells, domain spells, and spell slot calculations
- **Inventory** - Full equipment management, inventory items, gold, encumbrance, item property editing
- **Race** - Change race with subrace support and automatic stat adjustments
- **Appearance** - Customize body parts, phenotype, wings, tails, and colors with live 3D preview
- **Identity** - Name, biography, alignment, deity, age and gender
- **Combat Stats** - View attack bonuses, armor class, saving throws, and initiative

### Game State
- **Campaign Variables and Settings** - Edit global campaign settings and variables
- **Module Variables** - View and modify module-level state

### Save Management
- Automatic backup creation before every save
- Backup restore with safety snapshots

### Mod Support
- HAK pack override chain
- Steam Workshop integration
- Custom override directories

### 3D Model Viewer
- Browse and preview all in-game 3D models

## Requirements

- Windows 10+ / Ubuntu
- Neverwinter Nights 2 Enhanced Edition 

## Getting Started

1. Download the latest release
2. Run the `.exe` - game paths are auto-detected
3. Open a save file and start editing

## Building from Source

```bash
# Install dependencies
npm install

# Development
npm run tauri:dev

# Production build
npm run tauri:build
```

Requires:
- Node.js 18+
- Rust toolchain (stable)

## Roadmap

- Companion editing (view, edit, and manage party companions)
