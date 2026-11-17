# User research: what screenwriters complain about and ask for

Collected 2026-11-17 from public reviews, forums and vendor help pages about Final Draft,
WriterDuet, Fade In, Highland and Celtx (the apps [target-app-parity.md](target-app-parity.md)
measures against). Everything is paraphrased; no review text is reproduced. ScriptCraft learns
from what users report, not from any app's code, strings or feature names (AGENTS.md §2).

**How often** is a judgement from the sources found: *often* = raised across several apps or
sources; *sometimes* = more than one source; *once* = a single but credible report. Review samples
on G2, Capterra and similar sites are small (9 to 44 reviews per app), so this is a qualitative
picture, not a survey.

**Status:** *avoided* (ScriptCraft does not have the problem, by design or by what is built),
*have*, *planned* (in [gaps.md](gaps.md)), *rejected* (with the reason).

## Top annoyances

| # | Annoyance | How often | Source | ScriptCraft |
|---|---|---|---|---|
| 1 | Licence activation and reinstall limits; older paid versions stop activating, forcing a paid upgrade | often | [Rossmann wiki on Final Draft activation](https://wiki.rossmanngroup.com/wiki/Final_Draft_software_activation), [Final Draft upgrade page](https://www.finaldraft.com/products/final-draft-upgrade) | **avoided**: open source, no activation, no account, no expiry |
| 2 | Paid upgrades and subscription prices, especially for collaboration tiers | often | [G2 Final Draft reviews](https://www.g2.com/products/final-draft/reviews?page=2), [GetApp Celtx](https://www.getapp.co.uk/software/2047230/celtx), [Zoftwarehub Celtx](https://zoftwarehub.com/en-ae/products/celtx/reviews) | **avoided**: free (MIT OR Apache-2.0) |
| 3 | Work lost when cloud sync fails or an offline cache is cleared | sometimes | [WriterDuet help: write offline](https://writerduet.helpscoutdocs.com/article/234-write-offline), [Justuseapp WriterDuet reviews](https://justuseapp.com/en/app/1237820649/writerduet/reviews) | **avoided**: plain local files; atomic saves, autosave every minute with recovery, backup of the previous version |
| 4 | Scripts lost when a subscription is downgraded or lapses | once | [GetApp UK Celtx review](https://www.getapp.co.uk/software/2047230/celtx) | **avoided**: no subscription; Fountain files you own |
| 5 | Crashes, sometimes daily, and files that crash the app when opened | often | [r/Screenwriting "I hate Final Draft"](https://redlib.hackliberty.org/r/Screenwriting/comments/1jcpclv/i_hate_final_draft/mi409dn/?context=3), [r/Screenwriting FD13 crashing on opening](https://reddit.birdcat.cafe/r/Screenwriting/comments/1kzu44u/help_final_draft_13_crashing_on_opening_file), [AWN review](https://awn.com/blog/whats-best-screenwriting-software) | **avoided**: never-crash rules, readers that never fail on bad files (mutation-fuzz tested), panic guard keeps the script open |
| 6 | Files that will not open right before a deadline | sometimes | [r/Screenwriting](https://redlib.hackliberty.org/r/Screenwriting/comments/1jcpclv/i_hate_final_draft/mi409dn/?context=3) | **avoided**: the Fountain reader always opens text (anything unrecognised becomes action); backups and autosaves are plain Fountain too |
| 7 | Formatting silently drifts (dialogue jumping to the margin) with no warning | sometimes | [r/Screenwriting](https://redlib.hackliberty.org/r/Screenwriting/comments/1jcpclv/i_hate_final_draft/mi409dn/?context=3) | **have**: formatting is computed from element type on a fixed grid, never stored per paragraph; layout tests pin every column |
| 8 | Page count and margins slightly off from the standard | sometimes | [Zoftwarehub Celtx](https://zoftwarehub.com/en-ae/products/celtx/reviews), [John August on Highland line counts](https://johnaugust.com/apps/highland/page/16) | **have**: 54 lines on US Letter, 10 cpi, documented and tested ([formatting.md](formatting.md)) |
| 9 | Copy and paste between screenplay apps loses element types | sometimes | [G2 Highland 2 reviews](https://www.g2.com/products/highland-2/reviews) | **have** (this pass): copied elements are Fountain; pasted multi-paragraph text becomes typed elements |
| 10 | FDX import quirks: capitalization lost, page numbers imported as text | sometimes | [Storyist forum](https://forums.storyist.com/topic/3075-screenplays-template-issues-feature-parity-with-final-draft/) | **have** for elements and numbers; **planned**: styled text and script notes in FDX (gaps #2) |
| 11 | PDF export drops text at page bottoms or moves dialogue | once (older Highland) | [John August, Highland](https://johnaugust.com/apps/highland/page/16) | **have**: PDF pages are exactly the layout's pages, tested page for page |
| 12 | Stray Fountain markup ending up in FDX exports | once (fixed upstream) | [John August, Highland](https://johnaugust.com/apps/highland/page/11) | **have**: FDX is written from the model, never from Fountain text; round-trip tested |
| 13 | No Linux version | sometimes | [r/Screenwriting, operating systems](https://reddit.birdcat.cafe/r/Screenwriting/comments/1ddxo9h/people_who_hate_final_draft_which_operating) | **have**: Linux, macOS, Windows |
| 14 | Clunky upgrade and install process | sometimes | [G2 Final Draft](https://www.g2.com/products/final-draft/reviews?page=2) | **avoided**: one package per platform, no licence step |
| 15 | Mobile apps that are hard to use | sometimes | [Capterra Fade In Pro](https://www.capterra.com/p/214531/Fade-In-Pro/reviews/), [marlvel WriterDuet report](https://marlvel.ai/intel-report/productivity/writerduet) | **planned** later (web build first); not a priority for desktop writing |
| 16 | Sign-in and authentication failures blocking work | sometimes | [marlvel WriterDuet report](https://marlvel.ai/intel-report/productivity/writerduet) | **avoided**: no sign-in |
| 17 | Billing problems with memberships | once | [PissedConsumer WriterDuet](https://www.pissedconsumer.com/writerduet/RT-F.html) | **avoided**: nothing to bill |
| 18 | Thin support and help resources | sometimes | [pzaz.tv survey of filmmakers](https://pzaz.tv/producer-blog/best-film-screenwriting-software-usa) | **have**: keyboard help in the app, documented rules ([keyboard.md](keyboard.md), [formatting.md](formatting.md)); **planned**: a user guide |
| 19 | Occasional glitches that interrupt writing | sometimes | [G2 WriterDuet](https://www.g2.com/products/writerduet) | **have**: the editor draws only nearby lines, so a 90-page script takes about 1.3 ms a frame (was 53 ms before this pass) |
| 20 | Smaller template and integration ecosystem than the market leader | once | [Film Lifestyle on Fade In](https://filmlifestyle.com/store/product/fade-in) | **have**: open formats (Fountain, FDX, JSON scene list for BoardCraft), a CLI for automation |

## Most-requested features

| # | Request | How often | Source | ScriptCraft |
|---|---|---|---|---|
| 1 | Commenting and feedback on a script | often (76% of independent filmmakers call it essential in one analysis) | [pzaz.tv](https://pzaz.tv/producer-blog/best-film-screenwriting-software-usa) | **planned** (gaps #5): anchored notes; Fountain notes work today |
| 2 | Fountain export | often (59% essential in the same analysis) | [pzaz.tv](https://pzaz.tv/producer-blog/best-film-screenwriting-software-usa) | **have**: Fountain is the native format |
| 3 | FDX import and export that round-trips | often | [r/Screenwriting](https://redlib.hackliberty.org/r/Screenwriting/comments/1jcpclv/i_hate_final_draft/mi41fyz/?context=3), [Film Lifestyle](https://filmlifestyle.com/store/product/fade-in) | **have** (elements); **planned**: styles, notes |
| 4 | Real-time collaboration | often | [Film Lifestyle](https://filmlifestyle.com/store/product/fade-in), [G2 WriterDuet](https://www.g2.com/products/writerduet) | **planned** later (craft-contrib RFC-0020); local-first by design |
| 5 | AI help inside the writing app | sometimes (28% in one analysis) | [pzaz.tv](https://pzaz.tv/producer-blog/best-film-screenwriting-software-usa) | **planned**, opt-in, through the suite's shared AI layer; never required |
| 6 | Outlining tools | often | [Capterra screenwriting category](https://capterra.com/screenwriting-software), [Arc Studio blog](https://arcstudiopro.com/blog/6-best-screenwriting-software-of-2021-paid-and-free) | **have**: navigator, index cards with synopses; **planned**: section outline |
| 7 | Cloud backup | sometimes (21%) | [pzaz.tv](https://pzaz.tv/producer-blog/best-film-screenwriting-software-usa) | **rejected** as a built-in service (lock-in risk, see annoyances 3 and 4); files work in any sync folder, and backups are local |
| 8 | Works offline | often | [WriterDuet help](https://writerduet.helpscoutdocs.com/article/242-troubleshooting-tips) | **have**: entirely offline |
| 9 | Reliable PDF output for submissions | often | [John August](https://johnaugust.com/apps/highland/page/16) | **have** |
| 10 | Revision tracking with coloured pages | sometimes | [Final Draft upgrade page](https://www.finaldraft.com/products/final-draft-upgrade) | **have** |
| 11 | Locked scene numbers | sometimes | production practice | **have** |
| 12 | Index cards / visual story board | sometimes | [Capterra category](https://capterra.com/screenwriting-software) | **have** |
| 13 | Reports (characters, locations) | sometimes | [Capterra category](https://capterra.com/screenwriting-software) | **have** |
| 14 | Formatting that happens behind the scenes instead of as you type | once | [r/Screenwriting](https://redlib.hackliberty.org/r/Screenwriting/comments/1jcpclv/i_hate_final_draft/mi4ref4/?context=3) | **have**: type plain text (or paste Fountain); layout formats it |
| 15 | Linux support | sometimes | [r/Screenwriting](https://reddit.birdcat.cafe/r/Screenwriting/comments/1ddxo9h/people_who_hate_final_draft_which_operating) | **have** |
| 16 | Distraction-free writing | sometimes | [Capterra category](https://capterra.com/screenwriting-software) | **have**: focus mode, typewriter scrolling, dark page |
| 17 | A one-time price instead of a subscription | often | [Capterra Fade In Pro](https://www.capterra.com/p/214531/Fade-In-Pro/) | **avoided**: free |
| 18 | A mobile app that is pleasant to use | sometimes | [Capterra Fade In Pro](https://www.capterra.com/p/214531/Fade-In-Pro/reviews/) | **planned** later |
| 19 | Templates for other formats (TV, stage plays) | sometimes | [Film Lifestyle](https://filmlifestyle.com/store/product/fade-in) | **planned** (gaps: custom element styles) |
| 20 | Typing directly into FDX files | once | [r/Screenwriting](https://redlib.hackliberty.org/r/Screenwriting/comments/1gocfae/do_not_use_celtx/lwibu7v/?context=3) | **rejected** as the native format: Fountain is open and diffable; FDX open and export cover exchange |

## What this changed

The research moved two items to the top of this pass:

1. **Responsiveness on long scripts** (annoyance 19 and the polish-pass rule that stability comes
   first): the editor now draws only nearby lines. A 1,050-element script went from 53 ms to about
   1.3 ms per frame (release build), with a kittest test that a feature-length script draws fewer
   than 120 text fields and stays editable at its far end.
2. **Copy and paste between apps** (annoyance 9, and the top parity gap): whole elements are
   selected with Shift+↑/↓ at an element's edge, copied and cut as Fountain, and multi-paragraph
   paste becomes typed elements.

Data safety (annoyances 3 to 6) was already covered by the previous pass (atomic saves, autosave
with recovery, backups, never-crash readers); the research confirmed it as the right first
priority. Anchored comments (request 1) are now ranked high in [gaps.md](gaps.md).
