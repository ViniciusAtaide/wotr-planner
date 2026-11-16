//! Each option's progression header (ticket 304) against its blueprint's `BlueprintProgression` fields: the classes
//! and archetypes whose levels count with their offsets (`m_Classes`, `m_Archetypes`), half the other classes'
//! levels (`ForAllOtherClasses` with `AlternateProgressionType` `Div2`) and the rows of the levels before
//! (`GiveFeaturesForPreviousLevels`), as `BlueprintProgression.CalcLevel` and `LevelUpHelper.UpdateProgression` read
//! them (WrathDecompiled 6ce4a6b). An option whose blueprint is no progression has none.

use std::collections::BTreeMap;

use rules::{BlueprintGuid, DataAt, ProgressionHeader};
use serde_json::Value;

use super::Ctx;
use crate::dump::{flag, int, reference};
use crate::report::{Area, Out};

const AREA: Area = Area::ProgressionHeaders;

pub fn compare(ctx: &Ctx, out: &mut Out) {
    let data = ctx.planner.data;
    let classes: BTreeMap<&BlueprintGuid, &str> = (data.classes.iter())
        .chain(&data.prestige_classes)
        .filter_map(|class| Some((class.blueprint.as_ref()?, class.key.as_str())))
        .collect();
    let archetypes: BTreeMap<&BlueprintGuid, &str> = (data.archetypes.iter())
        .filter_map(|archetype| Some((archetype.blueprint.as_ref()?, archetype.key.as_str())))
        .collect();
    for option in &data.options {
        let at = DataAt::SelectionOption(option.key.clone());
        let Some((guid, json)) = ctx.blueprint(out, AREA, option.blueprint.as_ref(), "an option")
        else {
            continue;
        };
        let mark = out.mark();
        let is_progression = ctx.dump.kind(guid) == Some("BlueprintProgression");
        let game = is_progression.then(|| {
            Header {
                classes: offsets(&json["m_Classes"], "m_Class", |guid| {
                    classes.get(guid).copied()
                }),
                archetypes: offsets(&json["m_Archetypes"], "m_Archetype", |guid| {
                    archetypes.get(guid).copied()
                }),
                half_other_classes: flag(&json["ForAllOtherClasses"])
                    && json["AlternateProgressionType"].as_str() == Some("Div2"),
                previous_levels: flag(&json["GiveFeaturesForPreviousLevels"]),
            }
            .text(ctx)
        });
        let planner = option
            .header
            .as_ref()
            .map(|header| Header::of(header).text(ctx));
        if game != planner {
            out.finding(
                AREA,
                "a progression header",
                at.to_string(),
                guid,
                &ctx.name(guid),
                game.unwrap_or_else(|| "no progression".to_owned()),
                planner.unwrap_or_else(|| "no header".to_owned()),
            );
        }
        out.compared(AREA, mark);
    }
}

/// A header with each class and archetype by the planner's key, or the game's GUID when the planner has none.
struct Header {
    classes: BTreeMap<Key, i64>,
    archetypes: BTreeMap<Key, i64>,
    half_other_classes: bool,
    previous_levels: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Key {
    Planner(String),
    Game(BlueprintGuid),
}

impl Header {
    fn of(header: &ProgressionHeader) -> Header {
        Header {
            classes: (header.classes.iter())
                .map(|(class, offset)| {
                    (Key::Planner(class.as_str().to_owned()), i64::from(*offset))
                })
                .collect(),
            archetypes: (header.archetypes.iter())
                .map(|(archetype, offset)| {
                    (
                        Key::Planner(archetype.as_str().to_owned()),
                        i64::from(*offset),
                    )
                })
                .collect(),
            half_other_classes: header.half_other_classes,
            previous_levels: header.previous_levels,
        }
    }

    /// The header as one line: `classes a+0 b-2; archetypes c+0; half the other classes; previous levels`.
    fn text(&self, ctx: &Ctx) -> String {
        let list = |map: &BTreeMap<Key, i64>| {
            map.iter()
                .map(|(key, offset)| {
                    let key = match key {
                        Key::Planner(key) => key.clone(),
                        Key::Game(guid) => format!("(not in the planner) {}", ctx.name(guid)),
                    };
                    format!("{key}{offset:+}")
                })
                .collect::<Vec<_>>()
                .join(" ")
        };
        let mut parts = vec![
            format!("classes {}", list(&self.classes)),
            format!("archetypes {}", list(&self.archetypes)),
        ];
        if self.half_other_classes {
            parts.push("half the other classes".to_owned());
        }
        if self.previous_levels {
            parts.push("previous levels".to_owned());
        }
        parts.join("; ")
    }
}

/// `m_Classes` or `m_Archetypes`: each reference (`inner`) with its `AdditionalLevel`, by the planner's key.
fn offsets<'k>(
    value: &Value,
    inner: &str,
    key: impl Fn(&BlueprintGuid) -> Option<&'k str>,
) -> BTreeMap<Key, i64> {
    (value.as_array().map_or(&[][..], Vec::as_slice).iter())
        .filter_map(|item| {
            let guid = reference(&item[inner])?;
            let key = key(&guid).map_or(Key::Game(guid), |key| Key::Planner(key.to_owned()));
            Some((key, int(&item["AdditionalLevel"])))
        })
        .collect()
}
