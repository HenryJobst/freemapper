# freemapper-courseplan answers on the gym symbol set v0.1 (2026-11-08; recommendations, user not yet decided)

Q1 course symbol size: ×1.5 like the map symbols (control circle ~9 mm at 1:200 = 1.8 m); course symbols paper-fixed (P); their number placement uses base scale = map scale. ×1.0 would be one table value.
Q3 control descriptions for gym equipment: courseplan defines them: column-D extension with ids tied to our map codes (e.g. `gym.1910` box, `gym.1901` mat), names DE/EN; text first, pictograms later by print. Needs stable codes and names from us.
Q3b 709 OCAD numbers: courseplan writes no OCAD (.ppen + IOF-XML); map barriers are 520/1222; run-only closures are their special objects; our OCAD export follows our map file (709.400) — fine.
Q4/Q7: blue mats fine for them (print decides readability). Danger area: setup-related = map content, orange 1980 in the setup copy; run-only = their purple special object; both may coexist; 1980/1980.1 are map content, not drawn by courseplan.
Q6 layer contract: yes — `hall`, `courts`, `equipment` stable; one .omap per setup differing only in `equipment`. Requests: write a fixed `map_kind` in freemapper.toml (they suggested "indoor"; we use "gym") so they don't take 1:200 for a schoolyard map; stable 19xx codes for `equipment` objects (needed for their stage-2 setup layer).
Equipment sizes: they cannot supply real ones; use manufacturer catalogues (cite) or measure a hall; keep "(typ.)" marks.
