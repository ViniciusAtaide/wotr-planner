//! Storing [`Prerequisite`] trees as `prerequisite_node` rows and reading them back.
//!
//! A group's children are a set, so no order is stored: [`Trees::get`] returns them in whatever order it reads
//! them, and [`load_game_data`](crate::load_game_data) makes every tree canonical with the rest of the game data.
//! Every node belongs to exactly one owner's tree, which [`Trees::finish`] checks once every owner has read its
//! tree.

use std::collections::HashMap;

use rules::{
    ArchetypeKey, ClassKey, DeityKey, FeatKey, FeatureKey, FeatureSelectionKey, MythicAbilityKey,
    MythicFeatKey, Parameter, Prerequisite, RaceKey, SelectionOptionKey, SkillKey, WeaponKey,
};
use sqlx::SqliteConnection;

use crate::columns::{
    NodeKind, ability_from, ability_text, alignment_from, alignment_text, armour_type_from,
    armour_type_text, corrupt, energy_from, energy_text, node_kind_from, node_kind_text,
    option_group_from, option_group_text, spell_school_from, spell_school_text, spell_type_from,
    spell_type_text, story_from, story_text, to_u8,
};

/// The columns of one `prerequisite_node` row, apart from its place in the tree.
struct Node {
    kind: NodeKind,
    feat_key: Option<String>,
    mythic_feat_key: Option<String>,
    mythic_ability_key: Option<String>,
    class_key: Option<String>,
    archetype_key: Option<String>,
    feature_key: Option<String>,
    selection_option_key: Option<String>,
    skill_key: Option<String>,
    race_key: Option<String>,
    weapon_key: Option<String>,
    ability: Option<&'static str>,
    spell_type: Option<&'static str>,
    alignment: Option<&'static str>,
    amount: Option<i16>,
    spell_school: Option<&'static str>,
    energy: Option<&'static str>,
    armour_type: Option<&'static str>,
    story: Option<&'static str>,
    selection_key: Option<String>,
    option_group: Option<&'static str>,
    deity_key: Option<String>,
}

impl Node {
    fn of(kind: NodeKind) -> Self {
        Self {
            kind,
            feat_key: None,
            mythic_feat_key: None,
            mythic_ability_key: None,
            class_key: None,
            archetype_key: None,
            feature_key: None,
            selection_option_key: None,
            skill_key: None,
            race_key: None,
            weapon_key: None,
            ability: None,
            spell_type: None,
            alignment: None,
            amount: None,
            spell_school: None,
            energy: None,
            armour_type: None,
            story: None,
            selection_key: None,
            option_group: None,
            deity_key: None,
        }
    }

    /// Fills the one column that holds `value`.
    fn with_value(self, value: &Parameter) -> Self {
        match value {
            Parameter::Skill(skill) => Self {
                skill_key: Some(skill.as_str().to_owned()),
                ..self
            },
            Parameter::Weapon(weapon) => Self {
                weapon_key: Some(weapon.as_str().to_owned()),
                ..self
            },
            Parameter::SpellSchool(school) => Self {
                spell_school: Some(spell_school_text(*school)),
                ..self
            },
            Parameter::Energy(energy) => Self {
                energy: Some(energy_text(*energy)),
                ..self
            },
            Parameter::ArmourType(armour) => Self {
                armour_type: Some(armour_type_text(*armour)),
                ..self
            },
            Parameter::Metamagic(_) | Parameter::Stat(_) | Parameter::Element(_) => {
                unreachable!(
                    "only a pick names a metamagic, stat or element (GameData::check: PickOnlyValue)"
                )
            }
        }
    }

    fn with_amount(self, amount: u8) -> Self {
        Self {
            amount: Some(amount.into()),
            ..self
        }
    }

    /// The node for `prerequisite` and its children, if it's a group.
    fn from_prerequisite(prerequisite: &Prerequisite) -> (Self, &[Prerequisite]) {
        use Prerequisite as P;
        let key = |key: &str| Some(key.to_owned());
        let node = match prerequisite {
            P::All(children) => return (Self::of(NodeKind::All), children),
            P::Any(children) => return (Self::of(NodeKind::Any), children),
            P::AtLeast(n, children) => {
                return (Self::of(NodeKind::AtLeast).with_amount(*n), children);
            }
            P::Not(child) => return (Self::of(NodeKind::Not), std::slice::from_ref(&**child)),
            P::Feat(feat) => Self {
                feat_key: key(feat.as_str()),
                ..Self::of(NodeKind::Feat)
            },
            P::FeatFor { feat, value } => Self {
                feat_key: key(feat.as_str()),
                ..Self::of(NodeKind::FeatFor)
            }
            .with_value(value),
            P::FeatForSameValue(feat) => Self {
                feat_key: key(feat.as_str()),
                ..Self::of(NodeKind::FeatForSameValue)
            },
            P::FeatIfValue { value, feat } => Self {
                feat_key: key(feat.as_str()),
                ..Self::of(NodeKind::FeatIfValue)
            }
            .with_value(value),
            P::MythicFeat(feat) => Self {
                mythic_feat_key: key(feat.as_str()),
                ..Self::of(NodeKind::MythicFeat)
            },
            P::MythicAbility(ability) => Self {
                mythic_ability_key: key(ability.as_str()),
                ..Self::of(NodeKind::MythicAbility)
            },
            P::ClassLevel { class, level } => Self {
                class_key: key(class.as_str()),
                ..Self::of(NodeKind::ClassLevel).with_amount(*level)
            },
            P::Archetype(archetype) => Self {
                archetype_key: key(archetype.as_str()),
                ..Self::of(NodeKind::Archetype)
            },
            P::Feature { feature, count } => Self {
                feature_key: key(feature.as_str()),
                ..Self::of(NodeKind::Feature).with_amount(*count)
            },
            P::SelectionOption(option) => Self {
                selection_option_key: key(option.as_str()),
                ..Self::of(NodeKind::SelectionOption)
            },
            P::SkillRanks { skill, ranks } => Self {
                skill_key: key(skill.as_str()),
                ..Self::of(NodeKind::SkillRanks).with_amount(*ranks)
            },
            P::AbilityScore { ability, score } => Self {
                ability: Some(ability_text(*ability)),
                ..Self::of(NodeKind::AbilityScore).with_amount(*score)
            },
            P::BaseAttackBonus(bab) => Self::of(NodeKind::BaseAttackBonus).with_amount(*bab),
            P::SpellLevel { spell_type, level } => Self {
                spell_type: Some(spell_type_text(*spell_type)),
                ..Self::of(NodeKind::SpellLevel).with_amount(*level)
            },
            P::CasterLevel(level) => Self::of(NodeKind::CasterLevel).with_amount(*level),
            P::CharacterLevel(level) => Self::of(NodeKind::CharacterLevel).with_amount(*level),
            P::MythicRank(rank) => Self::of(NodeKind::MythicRank).with_amount(*rank),
            P::Alignment(alignment) => Self {
                alignment: Some(alignment_text(*alignment)),
                ..Self::of(NodeKind::Alignment)
            },
            P::Race(race) => Self {
                race_key: key(race.as_str()),
                ..Self::of(NodeKind::Race)
            },
            P::Story(story) => Self {
                story: Some(story_text(*story)),
                ..Self::of(NodeKind::Story)
            },
            P::NoOptionFrom(selection) => Self {
                selection_key: key(selection.as_str()),
                ..Self::of(NodeKind::NoOptionFrom)
            },
            P::SpontaneousSpellLevel { spell_type, level } => Self {
                spell_type: Some(spell_type_text(*spell_type)),
                ..Self::of(NodeKind::SpontaneousSpellLevel).with_amount(*level)
            },
            P::NoOtherOptionOfGroup(group) => Self {
                option_group: Some(option_group_text(*group)),
                ..Self::of(NodeKind::NoOtherOptionOfGroup)
            },
            P::PickedThrough(selection) => Self {
                selection_key: key(selection.as_str()),
                ..Self::of(NodeKind::PickedThrough)
            },
            P::Deity(deity) => Self {
                deity_key: key(deity.as_str()),
                ..Self::of(NodeKind::Deity)
            },
            P::Atheist => Self::of(NodeKind::Atheist),
            P::Worships => Self::of(NodeKind::Worships),
            P::DeityOffers(domain) => Self {
                selection_option_key: key(domain.as_str()),
                ..Self::of(NodeKind::DeityOffers)
            },
        };
        (node, &[])
    }
}

/// Inserts the tree and returns its root id, for the owner's `prerequisite_root_id`.
pub(crate) async fn insert(
    conn: &mut SqliteConnection,
    prerequisite: Option<&Prerequisite>,
) -> Result<Option<i64>, sqlx::Error> {
    let Some(root) = prerequisite else {
        return Ok(None);
    };
    let mut root_id = None;
    // Pre-order, so each parent is inserted (and has an id) before its children.
    let mut pending: Vec<(&Prerequisite, Option<i64>)> = vec![(root, None)];
    while let Some((prerequisite, parent_id)) = pending.pop() {
        let (node, children) = Node::from_prerequisite(prerequisite);
        let id = sqlx::query_scalar!(
            "INSERT INTO prerequisite_node (
                parent_id, kind, feat_key, mythic_feat_key, mythic_ability_key,
                class_key, archetype_key, feature_key, selection_option_key, skill_key, race_key,
                weapon_key, ability, spell_type, alignment, amount, spell_school, energy, armour_type,
                story, selection_key, option_group, deity_key
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17,
                $18, $19, $20, $21, $22, $23)
            RETURNING id AS \"id!\"",
            parent_id,
            node_kind_text(node.kind),
            node.feat_key,
            node.mythic_feat_key,
            node.mythic_ability_key,
            node.class_key,
            node.archetype_key,
            node.feature_key,
            node.selection_option_key,
            node.skill_key,
            node.race_key,
            node.weapon_key,
            node.ability,
            node.spell_type,
            node.alignment,
            node.amount,
            node.spell_school,
            node.energy,
            node.armour_type,
            node.story,
            node.selection_key,
            node.option_group,
            node.deity_key,
        )
        .fetch_one(&mut *conn)
        .await?;
        root_id.get_or_insert(id);
        for child in children.iter().rev() {
            pending.push((child, Some(id)));
        }
    }
    Ok(root_id)
}

/// A stored row, before it's turned back into a [`Prerequisite`].
struct StoredNode {
    parent_id: Option<i64>,
    kind: String,
    feat_key: Option<String>,
    mythic_feat_key: Option<String>,
    mythic_ability_key: Option<String>,
    class_key: Option<String>,
    archetype_key: Option<String>,
    feature_key: Option<String>,
    selection_option_key: Option<String>,
    skill_key: Option<String>,
    race_key: Option<String>,
    weapon_key: Option<String>,
    ability: Option<String>,
    spell_type: Option<String>,
    alignment: Option<String>,
    amount: Option<i64>,
    spell_school: Option<String>,
    energy: Option<String>,
    armour_type: Option<String>,
    story: Option<String>,
    selection_key: Option<String>,
    option_group: Option<String>,
    deity_key: Option<String>,
}

impl StoredNode {
    /// The value of a `feat_for` or `feat_if_value` node: whichever value column is set (the CHECK allows exactly
    /// one).
    fn value(&self) -> Option<Result<Parameter, sqlx::Error>> {
        if let Some(skill) = &self.skill_key {
            return Some(Ok(Parameter::Skill(SkillKey(skill.clone()))));
        }
        if let Some(weapon) = &self.weapon_key {
            return Some(Ok(Parameter::Weapon(WeaponKey(weapon.clone()))));
        }
        if let Some(school) = &self.spell_school {
            return Some(spell_school_from(school).map(Parameter::SpellSchool));
        }
        if let Some(energy) = &self.energy {
            return Some(energy_from(energy).map(Parameter::Energy));
        }
        self.armour_type
            .as_ref()
            .map(|armour| armour_type_from(armour).map(Parameter::ArmourType))
    }
}

/// Every stored prerequisite tree, to look up by root id.
pub(crate) struct Trees {
    nodes: HashMap<i64, StoredNode>,
    /// Child ids, in no particular order.
    children: HashMap<i64, Vec<i64>>,
    /// How many times [`get`](Self::get) has reached each node.
    reached: HashMap<i64, usize>,
}

impl Trees {
    pub(crate) async fn load(conn: &mut SqliteConnection) -> Result<Self, sqlx::Error> {
        let rows = sqlx::query!(
            "SELECT id, parent_id, kind, feat_key, mythic_feat_key, mythic_ability_key, class_key,
                archetype_key, feature_key, selection_option_key, skill_key, race_key, weapon_key,
                ability, spell_type, alignment, amount, spell_school, energy, armour_type, story,
                selection_key, option_group, deity_key
            FROM prerequisite_node
            ORDER BY id"
        )
        .fetch_all(&mut *conn)
        .await?;

        let mut trees = Self {
            nodes: HashMap::with_capacity(rows.len()),
            children: HashMap::new(),
            reached: HashMap::new(),
        };
        for row in rows {
            if let Some(parent_id) = row.parent_id {
                trees.children.entry(parent_id).or_default().push(row.id);
            }
            trees.nodes.insert(
                row.id,
                StoredNode {
                    parent_id: row.parent_id,
                    kind: row.kind,
                    feat_key: row.feat_key,
                    mythic_feat_key: row.mythic_feat_key,
                    mythic_ability_key: row.mythic_ability_key,
                    class_key: row.class_key,
                    archetype_key: row.archetype_key,
                    feature_key: row.feature_key,
                    selection_option_key: row.selection_option_key,
                    skill_key: row.skill_key,
                    race_key: row.race_key,
                    weapon_key: row.weapon_key,
                    ability: row.ability,
                    spell_type: row.spell_type,
                    alignment: row.alignment,
                    amount: row.amount,
                    spell_school: row.spell_school,
                    energy: row.energy,
                    armour_type: row.armour_type,
                    story: row.story,
                    selection_key: row.selection_key,
                    option_group: row.option_group,
                    deity_key: row.deity_key,
                },
            );
        }
        Ok(trees)
    }

    /// The tree rooted at `root_id`, or `None` when the owner has no prerequisite. A root has no parent.
    pub(crate) fn get(
        &mut self,
        root_id: Option<i64>,
    ) -> Result<Option<Prerequisite>, sqlx::Error> {
        let Some(root_id) = root_id else {
            return Ok(None);
        };
        let root = self
            .nodes
            .get(&root_id)
            .ok_or_else(|| corrupt(format!("prerequisite_node {root_id} doesn't exist")))?;
        if let Some(parent_id) = root.parent_id {
            // Descending from a parentless root can't loop: every node below it has one parent.
            return Err(corrupt(format!(
                "prerequisite_node {root_id} is an owner's root but has a parent ({parent_id})"
            )));
        }
        let mut reached = Vec::new();
        let tree = self.tree(root_id, &mut reached)?;
        for id in reached {
            *self.reached.entry(id).or_default() += 1;
        }
        Ok(Some(tree))
    }

    /// Checks that every node was reached exactly once: no tree is orphaned or shared by two owners.
    pub(crate) fn finish(self) -> Result<(), sqlx::Error> {
        let mut ids: Vec<i64> = self.nodes.keys().copied().collect();
        ids.sort_unstable();
        for id in ids {
            match self.reached.get(&id).copied().unwrap_or(0) {
                1 => {}
                0 => {
                    return Err(corrupt(format!(
                        "prerequisite_node {id} belongs to no owner"
                    )));
                }
                owners => {
                    return Err(corrupt(format!(
                        "prerequisite_node {id} belongs to {owners} owners"
                    )));
                }
            }
        }
        Ok(())
    }

    /// The tree at `id`, adding every node it reaches to `reached`.
    fn tree(&self, id: i64, reached: &mut Vec<i64>) -> Result<Prerequisite, sqlx::Error> {
        use Prerequisite as P;
        reached.push(id);
        let node = self
            .nodes
            .get(&id)
            .ok_or_else(|| corrupt(format!("prerequisite_node {id} doesn't exist")))?;
        let missing = |column: &str| corrupt(format!("prerequisite_node {id}: {column} is NULL"));
        let text =
            |value: &Option<String>, column: &str| value.clone().ok_or_else(|| missing(column));
        let amount = || {
            node.amount
                .ok_or_else(|| missing("amount"))
                .and_then(|amount| to_u8(amount, "prerequisite_node.amount"))
        };
        let mut children = || -> Result<Vec<Prerequisite>, sqlx::Error> {
            self.children
                .get(&id)
                .map(Vec::as_slice)
                .unwrap_or_default()
                .iter()
                .map(|&child| self.tree(child, reached))
                .collect()
        };

        let kind = node_kind_from(&node.kind)?;
        if !kind.is_group() && self.children.contains_key(&id) {
            return Err(corrupt(format!(
                "prerequisite_node {id}: a {} condition has children",
                node.kind
            )));
        }
        Ok(match kind {
            NodeKind::All => P::All(children()?),
            NodeKind::Any => P::Any(children()?),
            NodeKind::AtLeast => P::AtLeast(amount()?, children()?),
            NodeKind::Not => match <[Prerequisite; 1]>::try_from(children()?) {
                Ok([child]) => P::Not(Box::new(child)),
                Err(children) => {
                    return Err(corrupt(format!(
                        "prerequisite_node {id}: a not has {} children, not one",
                        children.len()
                    )));
                }
            },
            NodeKind::Feat => P::Feat(FeatKey(text(&node.feat_key, "feat_key")?)),
            NodeKind::MythicFeat => P::MythicFeat(MythicFeatKey(text(
                &node.mythic_feat_key,
                "mythic_feat_key",
            )?)),
            NodeKind::MythicAbility => P::MythicAbility(MythicAbilityKey(text(
                &node.mythic_ability_key,
                "mythic_ability_key",
            )?)),
            NodeKind::ClassLevel => P::ClassLevel {
                class: ClassKey(text(&node.class_key, "class_key")?),
                level: amount()?,
            },
            NodeKind::Archetype => {
                P::Archetype(ArchetypeKey(text(&node.archetype_key, "archetype_key")?))
            }
            NodeKind::Feature => P::Feature {
                feature: FeatureKey(text(&node.feature_key, "feature_key")?),
                count: amount()?,
            },
            NodeKind::SelectionOption => P::SelectionOption(SelectionOptionKey(text(
                &node.selection_option_key,
                "selection_option_key",
            )?)),
            NodeKind::SkillRanks => P::SkillRanks {
                skill: SkillKey(text(&node.skill_key, "skill_key")?),
                ranks: amount()?,
            },
            NodeKind::AbilityScore => P::AbilityScore {
                ability: ability_from(&text(&node.ability, "ability")?)?,
                score: amount()?,
            },
            NodeKind::BaseAttackBonus => P::BaseAttackBonus(amount()?),
            NodeKind::SpellLevel => P::SpellLevel {
                spell_type: spell_type_from(&text(&node.spell_type, "spell_type")?)?,
                level: amount()?,
            },
            NodeKind::CasterLevel => P::CasterLevel(amount()?),
            NodeKind::CharacterLevel => P::CharacterLevel(amount()?),
            NodeKind::MythicRank => P::MythicRank(amount()?),
            NodeKind::Alignment => {
                P::Alignment(alignment_from(&text(&node.alignment, "alignment")?)?)
            }
            NodeKind::Race => P::Race(RaceKey(text(&node.race_key, "race_key")?)),
            NodeKind::FeatFor => P::FeatFor {
                feat: FeatKey(text(&node.feat_key, "feat_key")?),
                value: node.value().ok_or_else(|| missing("the value"))??,
            },
            NodeKind::FeatForSameValue => {
                P::FeatForSameValue(FeatKey(text(&node.feat_key, "feat_key")?))
            }
            NodeKind::FeatIfValue => P::FeatIfValue {
                value: node.value().ok_or_else(|| missing("the value"))??,
                feat: FeatKey(text(&node.feat_key, "feat_key")?),
            },
            NodeKind::Story => P::Story(story_from(&text(&node.story, "story")?)?),
            NodeKind::NoOptionFrom => P::NoOptionFrom(FeatureSelectionKey(text(
                &node.selection_key,
                "selection_key",
            )?)),
            NodeKind::SpontaneousSpellLevel => P::SpontaneousSpellLevel {
                spell_type: spell_type_from(&text(&node.spell_type, "spell_type")?)?,
                level: amount()?,
            },
            NodeKind::NoOtherOptionOfGroup => P::NoOtherOptionOfGroup(option_group_from(&text(
                &node.option_group,
                "option_group",
            )?)?),
            NodeKind::PickedThrough => P::PickedThrough(FeatureSelectionKey(text(
                &node.selection_key,
                "selection_key",
            )?)),
            NodeKind::Deity => P::Deity(DeityKey(text(&node.deity_key, "deity_key")?)),
            NodeKind::Atheist => P::Atheist,
            NodeKind::Worships => P::Worships,
            NodeKind::DeityOffers => P::DeityOffers(SelectionOptionKey(text(
                &node.selection_option_key,
                "selection_option_key",
            )?)),
        })
    }
}
