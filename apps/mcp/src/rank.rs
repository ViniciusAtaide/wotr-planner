//! Ranking by a named number (ticket 127): `compare`'s `by` orders Builds by one number of a line the engine computes,
//! and `improve` tries every open option of one choice and orders them by the change each makes to that number. The
//! number is a fact (the attack line's or the caster line's, unconditional numbers only), so ranking by it is too; what
//! it leaves out is the line's own `leaves_out`.

use rules::{Address, Build, Engine, NotAnAddress, Sheet, SpellTraits, WeaponKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::attack;
use crate::caster::{self, CasterArgs};
use crate::code::open;
use crate::error::ToolError;
use crate::options::offers;
use crate::planner::Planner;
use crate::shapes::ChoiceValue;

/// A number to rank by: one of the caster line's or one of the attack line's, at the Build's last level.
#[derive(Clone, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum NumberArgs {
    /// A number of the caster line for these spells (`{"school": "enchantment", "subschool": "compulsion"}`).
    CasterLine(CasterNumberArgs),
    /// A number of the attack line for this base weapon (`{"weapon": "greatsword"}`), or without one for the
    /// Build's Weapon Focus weapon.
    AttackLine(AttackNumberArgs),
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct CasterNumberArgs {
    #[serde(flatten)]
    pub caster: CasterArgs,
    /// Which number (default `dc`): `dc`, the save DC of the highest spell level castable (the heightened row's when
    /// the Build holds Heighten Spell: the same DC); `spell_penetration`, caster level against spell resistance;
    /// `concentration`, the concentration bonus.
    #[serde(default)]
    pub number: CasterNumber,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CasterNumber {
    #[default]
    Dc,
    SpellPenetration,
    Concentration,
}

#[derive(Clone, Debug, Deserialize, JsonSchema)]
pub struct AttackNumberArgs {
    /// A base weapon's key (`greatsword`). Left out: `compare`'s `weapon`, else each Build's Weapon Focus weapon (its
    /// earliest Weapon Focus by level; within a level, the first weapon key); for `improve`, the Build's, which every
    /// option must keep (refused when the Build has none or an option would change it).
    #[serde(default)]
    pub weapon: Option<String>,
    /// Which number (default `attack`): `attack`, the first attack's bonus; `damage`, the bonus each hit adds to the
    /// weapon's dice (Power Attack not taken).
    #[serde(default)]
    pub number: AttackNumber,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AttackNumber {
    #[default]
    Attack,
    Damage,
}

/// A number, checked: what it is read from.
#[derive(Clone, Debug)]
pub(crate) enum Number {
    Caster(SpellTraits, CasterNumber),
    /// The attack line's number for a weapon; for each Build's Weapon Focus weapon when `None` (`compare` without
    /// `weapon`).
    Attack(Option<WeaponKey>, AttackNumber),
}

impl Number {
    /// The number an argument names, refused as `caster` or `weapon` would be.
    pub(crate) fn from_args(engine: &Engine, args: &NumberArgs) -> Result<Self, ToolError> {
        Ok(match args {
            NumberArgs::CasterLine(args) => {
                let spell = caster::caster_argument(Some(&args.caster))?
                    .expect("a caster argument given gives a spell");
                Self::Caster(spell, args.number)
            }
            NumberArgs::AttackLine(args) => Self::Attack(
                attack::weapon_argument(engine, args.weapon.as_deref())?,
                args.number,
            ),
        })
    }

    /// The number with this weapon for an attack line that names none: `compare`'s `weapon`, so the Builds are
    /// ranked by the line they show, or `improve`'s pinned weapon.
    pub(crate) fn or_weapon(self, weapon: Option<WeaponKey>) -> Self {
        match self {
            Self::Attack(None, number) => Self::Attack(weapon, number),
            number => number,
        }
    }

    /// The number at a Build's last level; `None` when it has no such line there (no level, no casting class, …).
    pub(crate) fn of(&self, engine: &Engine, build: &Build, sheet: &Sheet) -> Option<i16> {
        let last = &sheet.levels.last()?.state;
        match self {
            Self::Caster(spell, number) => {
                let line = engine.caster_line(&build.start, last, *spell)?;
                match number {
                    CasterNumber::Dc => {
                        line.heightened.as_ref().or(line.dcs.last()).map(|dc| dc.dc)
                    }
                    CasterNumber::SpellPenetration => Some(line.spell_penetration),
                    CasterNumber::Concentration => Some(line.concentration),
                }
            }
            Self::Attack(weapon, number) => {
                let weapon = attack::weapon_or_default(engine, weapon.as_ref(), sheet)?;
                let line = engine.attack_line(last, &weapon)?;
                match number {
                    AttackNumber::Attack => line.attacks.first().copied(),
                    AttackNumber::Damage => Some(line.damage_bonus),
                }
            }
        }
    }

    /// What the number is, in words.
    pub(crate) fn describe(&self, engine: &Engine) -> String {
        match self {
            Self::Caster(spell, number) => {
                let mut spells = spell.school.name().to_owned();
                if let Some(subschool) = spell.subschool {
                    spells = format!("{spells} ({})", subschool.name().to_lowercase());
                }
                if let Some(energy) = spell.energy {
                    spells = format!("{spells} {}", energy.name().to_lowercase());
                }
                let metamagic = if spell.metamagic {
                    ", with metamagic"
                } else {
                    ""
                };
                match number {
                    CasterNumber::Dc => format!(
                        "the save DC of {spells} spells of the highest level castable{metamagic}, at the last level"
                    ),
                    CasterNumber::SpellPenetration => format!(
                        "caster level against spell resistance, casting {spells} spells{metamagic}, at the last level"
                    ),
                    CasterNumber::Concentration => format!(
                        "the concentration bonus, casting {spells} spells{metamagic}, at the last level"
                    ),
                }
            }
            Self::Attack(weapon, number) => {
                let weapon = match weapon {
                    Some(weapon) => format!("a {}", engine.weapon_name(weapon)),
                    None => "each Build's Weapon Focus weapon".to_owned(),
                };
                match number {
                    AttackNumber::Attack => {
                        format!("the first attack's bonus with {weapon}, at the last level")
                    }
                    AttackNumber::Damage => format!(
                        "the bonus each hit with {weapon} adds to its dice (Power Attack not taken), at the last \
                         level"
                    ),
                }
            }
        }
    }

    /// What the line the number is read from leaves out.
    pub(crate) fn leaves_out(&self) -> Vec<String> {
        match self {
            Self::Caster(..) => caster::leaves_out(),
            Self::Attack(..) => attack::leaves_out(),
        }
    }
}

/// What `compare` ranked its Builds by.
#[derive(Debug, Serialize, JsonSchema)]
pub struct RankedBy {
    /// The number, in words.
    pub number: String,
    /// How the summaries are ordered.
    pub order: &'static str,
    /// What the line the number is read from leaves out: the number counts none of it.
    pub leaves_out: Vec<String>,
}

/// How `compare` orders Builds by a number.
pub(crate) const ORDER: &str = "highest number first; a Build without the line last; equal numbers keep the order \
    given";

/// The order of Builds by their numbers: indexes into `numbers`, highest first, `None` last, ties as given.
pub(crate) fn order(numbers: &[Option<i16>]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..numbers.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(numbers[i]));
    order
}

/// `improve`'s arguments.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ImproveArgs {
    /// The Build's code (a link works too).
    pub build: String,
    /// The choice's address, as `sheet`, `todo` and `options` take it: `slot.mythic.4`, `level.20.class`,
    /// `level.12.increase`, …
    pub choice: String,
    /// The number to rank the options by: `{"caster_line": {"school": "enchantment", "subschool": "compulsion"}}`
    /// (`number`: `dc`, the default, `spell_penetration` or `concentration`) or `{"attack_line": {"weapon":
    /// "greatsword"}}` (`weapon` left out: the Build's Weapon Focus weapon, refused when the Build has none or an
    /// option would change it; `number`: `attack`, the default, or `damage`).
    pub by: NumberArgs,
    /// Only options whose name or value holds every word of this (case doesn't matter), as `options` filters them.
    #[serde(default)]
    pub query: Option<String>,
}

/// `improve`'s result.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ImproveResult {
    /// The address, as the server writes it.
    pub choice: String,
    /// The number, in words.
    pub number: String,
    /// The Build's number now; absent when it has no such line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<i16>,
    /// The open options that change the number, highest number first (equal numbers in the planner's order).
    pub options: Vec<ImproveLine>,
    /// The names of the open options that leave the number as it is (the one held now among them), in the planner's
    /// order. Their effects, if any, are beyond the number: conditional ones (rolling twice, a gaze) are in their
    /// summaries.
    pub no_change: Vec<String>,
    /// The names of the open options whose Build has no such line at its last level.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub no_line: Vec<String>,
    /// How many options can't be taken now (`options` with `only_open: false` gives their reasons); none is tried.
    pub closed: u32,
    /// What the line the number is read from leaves out: the number counts none of it.
    pub leaves_out: Vec<String>,
}

/// One option that changes the number.
#[derive(Debug, Serialize, JsonSchema)]
pub struct ImproveLine {
    /// What `choose` takes for it.
    pub value: ChoiceValue,
    pub name: String,
    /// The number with this option taken.
    pub number: i16,
    /// The number with this option taken minus the number now; absent when the Build has no such line now.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change: Option<i16>,
}

/// `improve`: every open option of a choice taken by the planner's own edit, one Build each, ordered by the number
/// each Build has at its last level.
pub(crate) fn improve(planner: &Planner, args: ImproveArgs) -> Result<ImproveResult, ToolError> {
    let engine = &planner.engine;
    let build = open(planner, &args.build)?;
    let address: Address = args
        .choice
        .parse()
        .map_err(|error: NotAnAddress| ToolError::no_such_choice(error.to_string()))?;
    let sheet = engine.evaluate(&build);
    let number = Number::from_args(engine, &args.by)?;
    // An attack line that names no weapon reads the Build's Weapon Focus weapon, which every option tried must keep:
    // otherwise options would be read for different weapons, or for one their own sheets don't show.
    let kept = match number {
        Number::Attack(None, _) => Some(engine.default_attack_weapon(&sheet).ok_or_else(|| {
            ToolError::invalid_arguments(
                "`by`'s `attack_line` names no `weapon`, and the Build has no Weapon Focus whose weapon it could \
                 read. Give `weapon` to read every option for one weapon.",
            )
        })?),
        _ => None,
    };
    let number = number.or_weapon(kept.clone());
    let current = number.of(engine, &build, &sheet);
    let words: Vec<String> = args
        .query
        .as_deref()
        .unwrap_or_default()
        .split_whitespace()
        .map(str::to_lowercase)
        .collect();

    let (mut options, mut no_change, mut no_line, mut closed) =
        (Vec::new(), Vec::new(), Vec::new(), 0);
    for offer in offers(engine, &build, &sheet, &address, true)? {
        let text = offer.search_text();
        if !words.iter().all(|word| text.contains(word.as_str())) {
            continue;
        }
        let (Ok(()), Some(edit)) = (&offer.status, &offer.edit) else {
            closed += 1;
            continue;
        };
        // The option held now changes nothing, line or none.
        if offer.current {
            no_change.push(offer.name);
            continue;
        }
        let mut next = build.clone();
        engine.apply(&mut next, edit);
        let next_sheet = engine.evaluate(&next);
        if let Some(kept) = &kept {
            let after = engine.default_attack_weapon(&next_sheet);
            if after.as_ref() != Some(kept) {
                let after = after.map_or("none".to_owned(), |weapon| {
                    format!("the {}", engine.weapon_name(&weapon))
                });
                return Err(ToolError::invalid_arguments(format!(
                    "`by`'s `attack_line` names no `weapon`, so it reads the Build's Weapon Focus weapon, the {}; \
                     but {} at {address} would change it to {after}, so the options would be read for different \
                     weapons. Give `weapon` to read every option for one weapon.",
                    engine.weapon_name(kept),
                    offer.name,
                )));
            }
        }
        match number.of(engine, &next, &next_sheet) {
            None => no_line.push(offer.name),
            Some(after) if Some(after) == current => no_change.push(offer.name),
            Some(after) => options.push(ImproveLine {
                value: offer.value,
                name: offer.name,
                number: after,
                change: current.map(|now| after - now),
            }),
        }
    }
    options.sort_by_key(|line| std::cmp::Reverse(line.number));
    Ok(ImproveResult {
        choice: address.to_string(),
        number: number.describe(engine),
        current,
        options,
        no_change,
        no_line,
        closed,
        leaves_out: number.leaves_out(),
    })
}
