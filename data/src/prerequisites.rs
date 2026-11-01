//! Storing [`Prerequisite`] trees as `prerequisite_node` rows and reading them back.
//!
//! A group's children are a set, so no order is stored; [`Trees::get`] returns canonical trees. Every node
//! belongs to exactly one owner's tree, which [`Trees::finish`] checks once every owner has read its tree.

use std::collections::HashMap;

use rules::{
    ArchetypeKey, ClassKey, FeatKey, FeatureKey, MythicAbilityKey, MythicFeatKey, Prerequisite,
    RaceKey, SelectionOptionKey, SkillKey,
};
use sqlx::PgConnection;

use crate::columns::{
    NodeKind, ability_from, ability_text, alignment_from, alignment_text, corrupt, node_kind_from,
    node_kind_text, spell_type_from, spell_type_text, to_u8,
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
    ability: Option<&'static str>,
    spell_type: Option<&'static str>,
    alignment: Option<&'static str>,
    amount: Option<i16>,
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
            ability: None,
            spell_type: None,
            alignment: None,
            amount: None,
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
            P::Feat(feat) => Self {
                feat_key: key(feat.as_str()),
                ..Self::of(NodeKind::Feat)
            },
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
        };
        (node, &[])
    }
}

/// Inserts the tree and returns its root id, for the owner's `prerequisite_root_id`.
pub(crate) async fn insert(
    conn: &mut PgConnection,
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
                ability, spell_type, alignment, amount
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id",
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
            node.ability,
            node.spell_type,
            node.alignment,
            node.amount,
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
    ability: Option<String>,
    spell_type: Option<String>,
    alignment: Option<String>,
    amount: Option<i16>,
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
    pub(crate) async fn load(conn: &mut PgConnection) -> Result<Self, sqlx::Error> {
        let rows = sqlx::query!(
            "SELECT id, parent_id, kind, feat_key, mythic_feat_key, mythic_ability_key, class_key,
                archetype_key, feature_key, selection_option_key, skill_key, race_key, ability,
                spell_type, alignment, amount
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
                    ability: row.ability,
                    spell_type: row.spell_type,
                    alignment: row.alignment,
                    amount: row.amount,
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
        let mut tree = self.tree(root_id, &mut reached)?;
        for id in reached {
            *self.reached.entry(id).or_default() += 1;
        }
        tree.canonicalize();
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
        })
    }
}
