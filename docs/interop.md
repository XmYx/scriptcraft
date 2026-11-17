# Interop: ScriptCraft → BoardCraft (and other Crafting Apps)

ScriptCraft hands scripts to sibling apps in two ways. Use whichever fits; both come from the same
model, so they never disagree.

## 1. Fountain (primary)

Plain `.fountain` text, as ScriptCraft saves it ([docs/fountain.md](fountain.md)). BoardCraft reads
it directly. The scene rules match BoardCraft's importer:

- A **scene** starts at a scene heading: a paragraph of one line starting with `INT`, `EXT`, `EST`,
  `INT./EXT`, `INT/EXT` or `I/E` followed by `.` or a space (any case), or a line forced with a
  leading `.`. A trailing `#12A#` is the scene number.
- Inside a scene, each action paragraph and each speech (character cue, optional parentheticals,
  dialogue) is one beat (one storyboard panel in BoardCraft).
- Not printed and not beats: the title page, `[[notes]]`, `/* omitted text */`, `#` sections, `=`
  synopses, `===` page breaks. Transitions and shots are beats of their own kind.

### Stable scene ids

*Script ▸ Add Stable Scene IDs* puts a note right after each heading that has none:

```fountain
INT. LIGHTHOUSE - LAMP ROOM - NIGHT

[[scene-id: 7f3a09c2b1d84e65]]
```

The id is 16 hex digits, random, and never changes when the scene is renamed, renumbered or
moved. Readers look for a note whose text starts with `scene-id:` among the notes and synopses
directly after the heading. Notes are not printed, and other Fountain apps keep them as notes. A
storyboard app keys its scenes by this id to update boards on re-import instead of duplicating
them; a scene without an id has none (`null`).

## 2. Scene list (JSON)

*File ▸ Export Scene List for Storyboards…*, or from the command line:

```sh
scriptcraft-cli scenes script.fountain --out script.scenes.json [--paper a4]
```

Pages and eighths are counted on the chosen paper (US Letter by default).

```json
{
  "format": "scriptcraft-scenes",
  "version": 1,
  "generator": "ScriptCraft 0.1.0",
  "title": "THE LANTERN KEEPER",
  "source": "the-lantern-keeper.fountain",
  "paper": "us-letter",
  "page_count": 3,
  "scenes": [
    {
      "index": 2,
      "number": "2",
      "id": null,
      "heading": "INT. LIGHTHOUSE - LAMP ROOM - CONTINUOUS",
      "setting": { "interior_exterior": "INT.", "location": "LIGHTHOUSE - LAMP ROOM", "time_of_day": "CONTINUOUS" },
      "page_start": 1,
      "page_end": 1,
      "length_eighths": 6,
      "synopsis": null,
      "characters": ["TOMAS", "MAYA"],
      "action": "MAYA OKAFOR (40s, oilskin coat, steady hands) trims a wick…\n\nMaya grabs the handset.",
      "beats": [
        { "kind": "action", "text": "MAYA OKAFOR (40s, …" },
        { "kind": "dialogue", "text": "North Point, this is the Gull. Do you read?", "character": "TOMAS", "cue": "TOMAS (V.O.)", "parenthetical": "(through static)" },
        { "kind": "shot", "text": "CLOSE ON THE CHART TABLE" },
        { "kind": "transition", "text": "CUT TO:" }
      ]
    }
  ]
}
```

| Field | Meaning |
|---|---|
| `format`, `version` | Always `scriptcraft-scenes`, `1`. A reader rejects other formats and other major versions and ignores fields it does not know. |
| `index` | 1-based position of the scene in the script. |
| `number` | The heading's locked scene number (`12A`), else `index` as text. |
| `id` | Stable id from `[[scene-id: …]]`, or `null`. |
| `heading` | The heading as printed (capitals). |
| `setting` | The heading split: `interior_exterior` (`INT.`, `EXT.`, `INT./EXT.`, `I/E`, `EST.` or `null`), `location`, `time_of_day` (text after the last ` - `, or `null`). |
| `page_start`, `page_end` | Pages the scene starts and ends on (1-based; the title page does not count). |
| `length_eighths` | Scene length in eighths of a page, at least 1. |
| `synopsis` | The first `=` synopsis in the scene, or `null`. |
| `characters` | Speaking characters in order of first line, without extensions (`(V.O.)`). |
| `action` | The scene's action paragraphs, separated by a blank line. |
| `beats` | Everything printed, in order. `kind` is `action`, `dialogue`, `lyric`, `shot`, `transition` or `centered`. Dialogue and lyric beats add `character`, `cue` (as printed) and, when present, `parenthetical`; `dual: true` marks dual dialogue. |

Text before the first scene heading (`FADE IN:`) is not part of any scene.

The Rust types are `scriptcraft_interchange::SceneList` and friends; `from_json` reads a list back
and checks the format and version.
