# Game data schema and seeding

Type: grilling
Status: open
Blocked by: 01, 04
Map: [WotR Build Planner](../map.md)

## Question

What are the relational Postgres schema and the checked-in data file format for game data, and how do the files get into the database?

- Tables for races, classes, class progression, archetypes, feats, mythic paths, mythic feats and abilities, feature selections and skills.
- **Relational prerequisite trees:** how nested AND/OR prerequisite expressions are stored without JSONB, and how they load into a Rust prerequisite enum.
- **Data file format:** RON, JSON or TOML, and the layout of the files (one per category, or one per entity).
- **Seed pipeline:** re-runnable after migrations, and idempotent or truncate-and-reload.
- **Stable IDs:** how game entities are identified so saved Builds don't break when the data is re-seeded.
