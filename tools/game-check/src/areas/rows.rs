//! What each level gives: progression rows of classes, prestige classes and archetypes, and each mythic path's
//! ranks.
//!
//! The game's row at a level is its progression's `LevelEntries` there. An archetype changes it as
//! `ProgressionData.CalculateLevelEntries` does: one occurrence taken out per `RemoveFeatures` item, then the
//! `AddFeatures` appended (WrathDecompiled 6ce4a6b). The planner's is the row in force ([`rules::effective_row`]).
//! Both become multisets of grants by level.
//!
//! Before comparing, the game's grants the planner doesn't map are unfolded: a selection becomes a pick of its
//! `Group`, which a feat slot of the planner's matches; a feature that adds facts becomes those facts; a nested
//! progression becomes its own rows at the levels it reaches; a hidden one is left out. The facts a mapped feature
//! adds are kept as optional: they match a planner grant at that level or go.
//!
//! Then the two sides are paired: the same grant at the same level; the same grant at another level; a game selection
//! and a planner selection of the same group at one level (the game's own copy of a selection the planner shares); a
//! game selection and a planner feat slot at one level. What's left is in one side only, or a further rank of a
//! feature the other side gives earlier (never of a selection or a pick: granted again, it is another pick).

use rules::{BlueprintGuid, Class, DataAt, FeatCategory, FeatSlot, effective_row};
use serde_json::Value;

use super::Ctx;
use super::fixed::{
    FIRST_ASCENSION_SELECTION, FIRST_LEVELS, MYTHIC_ABILITY_SELECTION, MYTHIC_FEAT_SELECTION,
    MYTHIC_STARTING_CLASS, MYTHIC_STARTING_PROGRESSION, guid,
};
use crate::dump::{
    component_kind, components, flag, int, is_feature_kind, level_entries, reference, references,
};
use crate::report::{Area, Out};

/// How deep the unfolding goes: a bundle in a nested progression in a row is 3.
const DEPTH: u8 = 4;

/// One grant: a blueprint, or a pick of a group (a game selection the planner doesn't map, or a planner feat slot).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Item {
    Guid(BlueprintGuid),
    Pick(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum How {
    Grant,
    /// A fact a mapped feature adds: it matches a planner grant at its level or goes.
    Optional,
    /// A planner feat slot whose list comes from a held option: it pairs with any game pick at its level.
    AnyPick,
}

/// A grant at a level (or rank), with what the report calls it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Placed {
    item: Item,
    level: i64,
    label: String,
    /// A selection's `Group`, or a slot's.
    group: Option<String>,
    how: How,
    /// A selection (the game's `BlueprintFeatureSelection`, or the planner's selection or feat list): granted
    /// again, it is another pick, never a further rank (`BlueprintFeature.MeetsPrerequisites` lets an
    /// `IFeatureSelection` through at any rank).
    choice: bool,
}

impl Placed {
    fn new(item: Item, level: i64, label: String) -> Placed {
        Placed {
            item,
            level,
            label,
            group: None,
            how: How::Grant,
            choice: false,
        }
    }

    fn is_pick(&self) -> bool {
        matches!(self.item, Item::Pick(_))
    }
}

/// A selection blueprint's `Group`: `None` for any other blueprint.
fn group_of(ctx: &Ctx, guid: &BlueprintGuid) -> Option<String> {
    (ctx.dump.kind(guid) == Some("BlueprintFeatureSelection"))
        .then(|| ctx.dump.json(guid))
        .flatten()
        .and_then(|json| json["Group"].as_str())
        .map(str::to_owned)
}

pub fn compare_classes(ctx: &Ctx, out: &mut Out) {
    const AREA: Area = Area::ProgressionRows;
    for class in ctx.planner.data.all_classes() {
        let Some(class_guid) = class.blueprint.as_ref() else {
            out.skip(AREA, "class with no blueprint");
            continue;
        };
        let Some(progression) = ctx
            .dump
            .json(class_guid)
            .and_then(|json| reference(&json["m_Progression"]))
            .and_then(|guid| ctx.dump.json(&guid))
        else {
            out.skip(AREA, "a class whose progression isn't in the dump");
            continue;
        };
        let levels = class.levels.len() as i64;
        let base = level_entries(&progression["LevelEntries"]);
        let unfold = Unfold {
            ctx,
            area: AREA,
            max: levels,
            levels: &|class: &BlueprintGuid, at: i64| if class == class_guid { at } else { 0 },
        };

        let mark = out.mark();
        let game = unfold.rows(
            out,
            (1..=levels).map(|level| (level, base.get(&level).cloned().unwrap_or_default())),
        );
        let planner = planner_class_rows(ctx, out, AREA, class, None);
        let at = DataAt::Class(class.key.clone());
        diff(ctx, out, AREA, "level", &at, class_guid, game, planner);
        out.compared(AREA, mark);

        for archetype in ctx.planner.data.archetypes_of(&class.key) {
            let Some((guid, json)) =
                ctx.blueprint(out, AREA, archetype.blueprint.as_ref(), "archetype")
            else {
                continue;
            };
            let add = level_entries(&json["AddFeatures"]);
            let remove = level_entries(&json["RemoveFeatures"]);
            let rows = (1..=levels).map(|level| {
                let mut row = base.get(&level).cloned().unwrap_or_default();
                for removed in remove.get(&level).into_iter().flatten() {
                    if let Some(index) = row.iter().position(|each| each == removed) {
                        row.remove(index);
                    }
                }
                row.extend(add.get(&level).into_iter().flatten().cloned());
                (level, row)
            });
            let mark = out.mark();
            let game = unfold.rows(out, rows);
            let planner = planner_class_rows(ctx, out, AREA, class, Some(archetype));
            let at = DataAt::Archetype(archetype.key.clone());
            diff(ctx, out, AREA, "level", &at, guid, game, planner);
            out.compared(AREA, mark);
        }
    }
}

pub fn compare_mythic(ctx: &Ctx, out: &mut Out) {
    const AREA: Area = Area::MythicRanks;
    let starting_class = guid(MYTHIC_STARTING_CLASS);
    let general = [
        MYTHIC_ABILITY_SELECTION,
        MYTHIC_FEAT_SELECTION,
        FIRST_ASCENSION_SELECTION,
    ]
    .map(guid);
    let starting = ctx
        .dump
        .json(&guid(MYTHIC_STARTING_PROGRESSION))
        .map(|json| level_entries(&json["LevelEntries"]))
        .unwrap_or_default();
    for path in &ctx.planner.data.mythic_paths {
        let Some((path_guid, class)) =
            ctx.blueprint(out, AREA, path.blueprint.as_ref(), "mythic path")
        else {
            continue;
        };
        let Some(own) = reference(&class["m_Progression"]).and_then(|guid| ctx.dump.json(&guid))
        else {
            out.skip(AREA, "a path whose progression isn't in the dump");
            continue;
        };
        let own = level_entries(&own["LevelEntries"]);
        let first_levels = FIRST_LEVELS
            .iter()
            .find(|(class, _)| guid(class) == *path_guid)
            .and_then(|(_, progression)| ctx.dump.json(&guid(progression)))
            .map(|json| level_entries(&json["LevelEntries"]))
            .unwrap_or_default();
        // The ranks before the path's class: its own `PrerequisiteClassLevel` of the starting class, or for a path
        // taken later (a switch), its `PrerequisiteMythicLevel`. Those ranks are the starting class's, plus the
        // path's first levels where it has them; a later path has none of its own there.
        let starting_levels = components(class)
            .iter()
            .filter(|component| {
                component_kind(component) == "PrerequisiteClassLevel" && !flag(&component["Not"])
            })
            .find(|component| {
                reference(&component["m_CharacterClass"]).as_ref() == Some(&starting_class)
            })
            .map_or(0, |component| int(&component["Level"]));
        let switch = components(class)
            .iter()
            .find(|component| component_kind(component) == "PrerequisiteMythicLevel")
            .map(|component| int(&component["Level"]));
        let before = switch.unwrap_or(starting_levels).max(starting_levels);
        let ranks = path
            .ranks
            .iter()
            .map(|row| i64::from(row.rank))
            .max()
            .unwrap_or(0);
        let levels = |class: &BlueprintGuid, rank: i64| {
            if *class == starting_class {
                rank.min(starting_levels)
            } else if class == path_guid {
                (rank - before).max(0)
            } else {
                0
            }
        };
        let unfold = Unfold {
            ctx,
            area: AREA,
            max: ranks,
            levels: &levels,
        };
        let rows = (1..=ranks).map(|rank| {
            let mut row: Vec<BlueprintGuid> = if switch.is_some() && rank <= before {
                Vec::new()
            } else if rank <= before {
                let mut row = starting.get(&rank).cloned().unwrap_or_default();
                row.extend(first_levels.get(&rank).into_iter().flatten().cloned());
                row
            } else {
                own.get(&(rank - before)).cloned().unwrap_or_default()
            };
            row.retain(|grant| !general.contains(grant));
            (rank, row)
        });
        let mark = out.mark();
        let game = unfold.rows(out, rows);
        let mut planner = Vec::new();
        for row in &path.ranks {
            let grants = Grants {
                features: &row.features,
                feats: &[],
                feats_for: &[],
                slots: &row.feat_slots,
                selections: &row.selections,
            };
            grants.place(ctx, out, AREA, i64::from(row.rank), &mut planner);
        }
        let at = DataAt::MythicPath(path.key.clone());
        diff(ctx, out, AREA, "rank", &at, path_guid, game, planner);
        out.compared(AREA, mark);
    }
}

/// The game's side: what unfolds from each row's grants.
struct Unfold<'a> {
    ctx: &'a Ctx<'a>,
    area: Area,
    /// The last level compared.
    max: i64,
    /// A class's level at a level of the rows (the class's own; for a path, the starting class's or the path's).
    levels: &'a dyn Fn(&BlueprintGuid, i64) -> i64,
}

impl Unfold<'_> {
    fn rows(
        &self,
        out: &mut Out,
        rows: impl Iterator<Item = (i64, Vec<BlueprintGuid>)>,
    ) -> Vec<Placed> {
        let mut placed = Vec::new();
        for (level, grants) in rows {
            for grant in grants {
                self.grant(out, &grant, level, 0, How::Grant, &mut placed);
            }
        }
        placed
    }

    fn grant(
        &self,
        out: &mut Out,
        grant: &BlueprintGuid,
        level: i64,
        depth: u8,
        how: How,
        placed: &mut Vec<Placed>,
    ) {
        let ctx = self.ctx;
        let name = ctx.name(grant);
        let keep = |placed: &mut Vec<Placed>| {
            let group = group_of(ctx, grant);
            placed.push(Placed {
                choice: group.is_some(),
                group,
                how,
                ..Placed::new(Item::Guid(grant.clone()), level, name.clone())
            })
        };
        let json = ctx.dump.json(grant);
        if ctx.planner.maps(grant) || depth >= DEPTH {
            keep(placed);
            for fact in json
                .map(|json| feature_facts(ctx, json))
                .unwrap_or_default()
            {
                self.grant(out, &fact, level, depth + 1, How::Optional, placed);
            }
            return;
        }
        let Some(json) = json else {
            return keep(placed);
        };
        match ctx.dump.kind(grant) {
            Some("BlueprintFeatureSelection") => {
                let group = json["Group"].as_str().unwrap_or("None").to_owned();
                placed.push(Placed {
                    group: Some(group.clone()),
                    how,
                    ..Placed::new(
                        Item::Pick(group.clone()),
                        level,
                        format!("a {group} selection ({name})"),
                    )
                });
                return;
            }
            Some("BlueprintProgression") => {
                return self.nested(out, json, level, depth, how, placed);
            }
            _ => {}
        }
        // A feature that adds features stands for them. It stands for itself too only when it does more than add
        // facts and the planner maps none of them (a wrapper of a feat the planner grants is that feat).
        let facts = feature_facts(ctx, json);
        for fact in &facts {
            self.grant(out, fact, level, depth + 1, how, placed);
        }
        let parts = components(json);
        let only_facts =
            !parts.is_empty() && parts.iter().all(|part| component_kind(part) == "AddFacts");
        if only_facts || facts.iter().any(|fact| ctx.planner.maps(fact)) {
            return;
        }
        if flag(&json["HideInUI"]) || flag(&json["HideInCharacterSheetAndLevelUp"]) {
            out.skip(self.area, "a hidden grant the planner doesn't map");
            return;
        }
        keep(placed);
    }

    /// A nested progression granted at `granted`: each of its rows at the first level from there where its level
    /// (its classes' levels, each with its `AdditionalLevel`, as `BlueprintProgression.CalcLevel` sums them) reaches it.
    fn nested(
        &self,
        out: &mut Out,
        json: &Value,
        granted: i64,
        depth: u8,
        how: How,
        placed: &mut Vec<Placed>,
    ) {
        let classes: Vec<(BlueprintGuid, i64)> = json["m_Classes"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|entry| {
                Some((
                    reference(&entry["m_Class"])?,
                    int(&entry["AdditionalLevel"]),
                ))
            })
            .collect();
        if classes.is_empty() {
            out.skip(self.area, "a nested progression by character level");
            return;
        }
        let level_at = |at: i64| -> i64 {
            classes
                .iter()
                .map(|(class, additional)| {
                    let level = (self.levels)(class, at);
                    if level > 0 {
                        (level + additional).max(0)
                    } else {
                        0
                    }
                })
                .sum()
        };
        for (row, grants) in level_entries(&json["LevelEntries"]) {
            let Some(at) = (granted..=self.max).find(|at| level_at(*at) >= row) else {
                continue;
            };
            for grant in grants {
                self.grant(out, &grant, at, depth + 1, how, placed);
            }
        }
    }
}

/// The features a feature adds (`AddFacts`), leaving out abilities, buffs and the like, which no row compares.
fn feature_facts(ctx: &Ctx, json: &Value) -> Vec<BlueprintGuid> {
    components(json)
        .iter()
        .filter(|part| component_kind(part) == "AddFacts")
        .flat_map(|part| references(part.get("m_Facts")))
        .filter(|fact| ctx.dump.kind(fact).is_some_and(is_feature_kind))
        .collect()
}

/// The planner's rows of a class, or of one of its archetypes: the row in force at each level.
fn planner_class_rows(
    ctx: &Ctx,
    out: &mut Out,
    area: Area,
    class: &Class,
    archetype: Option<&rules::Archetype>,
) -> Vec<Placed> {
    let mut placed = Vec::new();
    for level in 1..=class.levels.len() as u8 {
        let Some(row) = effective_row(class, archetype, level) else {
            continue;
        };
        let feats: Vec<_> = row.feats.iter().collect();
        let feats_for: Vec<_> = row.feats_for.iter().map(|value| &value.feat).collect();
        let grants = Grants {
            features: row.features,
            feats: &feats,
            feats_for: &feats_for,
            slots: row.feat_slots,
            selections: row.selections,
        };
        grants.place(ctx, out, area, i64::from(level), &mut placed);
    }
    placed
}

/// What one planner row grants.
struct Grants<'a> {
    features: &'a [rules::FeatureKey],
    feats: &'a [&'a rules::FeatKey],
    feats_for: &'a [&'a rules::FeatKey],
    slots: &'a [FeatSlot],
    selections: &'a [rules::FeatureSelectionKey],
}

impl Grants<'_> {
    fn place(&self, ctx: &Ctx, out: &mut Out, area: Area, level: i64, placed: &mut Vec<Placed>) {
        let entry = |at: DataAt, out: &mut Out, placed: &mut Vec<Placed>| match ctx
            .planner
            .guid(at.clone())
        {
            Some(guid) => {
                let group = group_of(ctx, guid);
                placed.push(Placed {
                    choice: group.is_some()
                        || matches!(at, DataAt::FeatureSelection(_) | DataAt::FeatList(_)),
                    group,
                    ..Placed::new(
                        Item::Guid(guid.clone()),
                        level,
                        at.key().unwrap_or("").to_owned(),
                    )
                });
            }
            None => out.skip(area, "a planner grant with no blueprint"),
        };
        for feature in self.features {
            entry(DataAt::Feature(feature.clone()), out, placed);
        }
        for feat in self.feats {
            entry(DataAt::Feat((*feat).clone()), out, placed);
        }
        for feat in self.feats_for {
            out.skip(area, "a feat granted for one value, compared as the feat");
            entry(DataAt::Feat((*feat).clone()), out, placed);
        }
        for selection in self.selections {
            entry(DataAt::FeatureSelection(selection.clone()), out, placed);
        }
        for slot in self.slots {
            let group = match slot {
                FeatSlot::OneOf { list } => {
                    entry(DataAt::FeatList(list.clone()), out, placed);
                    continue;
                }
                FeatSlot::OptionList { .. } => {
                    placed.push(Placed {
                        how: How::AnyPick,
                        ..Placed::new(
                            Item::Pick("an option's list".to_owned()),
                            level,
                            "an option-list feat slot".to_owned(),
                        )
                    });
                    continue;
                }
                FeatSlot::General => "Feat".to_owned(),
                FeatSlot::CombatFeat => "CombatFeat".to_owned(),
                FeatSlot::Category(FeatCategory::Teamwork) => "TeamworkFeat".to_owned(),
                FeatSlot::Category(FeatCategory::MountedCombat) => "MountedCombatFeat".to_owned(),
                // Inference: the game's bonus metamagic and item creation feats are the `WizardFeat` group
                // (`WizardFeatSelection`, `SkaldFeatSelection`, `SorcererBonusFeat`).
                FeatSlot::Category(FeatCategory::Magic) => "WizardFeat".to_owned(),
                FeatSlot::Category(category) => format!("{category:?} (no game group)"),
            };
            placed.push(Placed {
                group: Some(group.clone()),
                ..Placed::new(
                    Item::Pick(group.clone()),
                    level,
                    format!("a {group} feat slot"),
                )
            });
        }
    }
}

/// The two sides' grants, paired as the module says.
#[allow(clippy::too_many_arguments)]
fn diff(
    ctx: &Ctx,
    out: &mut Out,
    area: Area,
    unit: &str,
    at: &DataAt,
    guid: &BlueprintGuid,
    mut game: Vec<Placed>,
    mut planner: Vec<Placed>,
) {
    let name = ctx.name(guid);
    game.sort();
    planner.sort();
    let (game_all, planner_all) = (game.clone(), planner.clone());
    let show = |placed: &Placed| format!("{} at {unit} {}", placed.label, placed.level);

    // The same grant at the same level.
    game.retain(|placed| {
        match planner
            .iter()
            .position(|other| other.item == placed.item && other.level == placed.level)
        {
            Some(index) => {
                planner.remove(index);
                false
            }
            None => true,
        }
    });
    game.retain(|placed| {
        let optional = placed.how == How::Optional;
        if optional {
            out.skip(area, "a fact a mapped feature adds, unmatched");
        }
        !optional
    });

    let mut pair = |out: &mut Out,
                    kind: &'static str,
                    matches: &dyn Fn(&Placed, &Placed) -> bool,
                    game: &mut Vec<Placed>| {
        game.retain(
            |placed| match planner.iter().position(|other| matches(placed, other)) {
                Some(index) => {
                    let other = planner.remove(index);
                    if other.how == How::AnyPick {
                        out.skip(
                            area,
                            "an option-list feat slot, paired with the game's pick",
                        );
                    } else {
                        out.finding(area, kind, at, guid, &name, show(placed), show(&other));
                    }
                    false
                }
                None => true,
            },
        );
    };
    pair(
        out,
        "at another level",
        &|placed, other| other.item == placed.item,
        &mut game,
    );
    pair(
        out,
        "a selection the game keeps separate",
        &|placed, other| {
            placed.is_pick()
                && !other.is_pick()
                && other.level == placed.level
                && other.group.is_some()
                && other.group == placed.group
        },
        &mut game,
    );
    pair(
        out,
        "a bonus pick of another group",
        &|placed, other| placed.is_pick() && other.is_pick() && other.level == placed.level,
        &mut game,
    );

    for placed in game {
        let kind = if rank_of(&planner_all, &placed) {
            "a further rank in game only"
        } else {
            "in game only"
        };
        out.finding(area, kind, at, guid, &name, show(&placed), "-");
    }
    for placed in planner {
        if placed.how == How::AnyPick {
            // The held option's own rows grant its feat in the game (a bloodline's progression).
            out.skip(
                area,
                "an option-list feat slot the option's rows grant in the game",
            );
            continue;
        }
        let kind = if rank_of(&game_all, &placed) {
            "a further rank in planner only"
        } else {
            "in planner only"
        };
        out.finding(area, kind, at, guid, &name, "-", show(&placed));
    }
}

/// Whether the other side grants this feature at an earlier level: then this one is a further rank of it. A pick
/// or a selection granted again is another pick, so it never is.
fn rank_of(other: &[Placed], placed: &Placed) -> bool {
    !placed.is_pick()
        && !placed.choice
        && other
            .iter()
            .any(|each| each.item == placed.item && each.level < placed.level)
}
