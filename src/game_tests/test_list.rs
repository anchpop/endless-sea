use crate::{character_tests::*, movement_tests::*, terrain_gen_tests::*};

pub(crate) fn all_tests() -> Vec<(&'static str, fn())> {
    vec![
        ("flat_gen", flat_gen),
        ("simplex_gen", simplex_gen),
        ("scale_gen", scale_gen),
        ("lump_gen", lump_gen),
        ("terrace_gen", terrace_gen),
        ("complex_gen", complex_gen),
        ("character_moves_horizontally", character_moves_horizontally),
        (
            "character_doesnt_move_vertically",
            character_doesnt_move_vertically,
        ),
        ("create_world", create_world),
        ("character_dies", character_dies),
        ("sword_attack", sword_attack),
    ]
}
