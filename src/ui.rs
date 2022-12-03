use bevy::prelude::*;

use crate::{character::Inventory, player::Player};

// Components
// ==========

#[derive(Reflect, Component, Default, Clone)]
#[reflect(Component)]
pub struct InventoryUI;

// Bundle
// ======

// Plugin
// ======

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_ui)
            .add_system(update_inventory);
    }
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            // inventory
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(200.0), Val::Percent(30.0)),
                        border: UiRect::all(Val::Px(2.0)),

                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::rgba(0.1, 0.1, 0.1, 0.5).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(
                        TextBundle::from_section(
                            "Inventory",
                            TextStyle {
                                font: asset_server
                                    .load("fonts/FiraCode-Bold.ttf"),
                                font_size: 30.0,
                                color: Color::WHITE,
                            },
                        )
                        .with_style(Style { ..default() }),
                    );
                    parent.spawn((
                        TextBundle::from_sections([
                            TextSection::new(
                                "Items go here",
                                TextStyle {
                                    font: asset_server
                                        .load("fonts/FiraCode-regular.ttf"),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            ),
                            TextSection::new(
                                "\n",
                                TextStyle {
                                    font: asset_server
                                        .load("fonts/FiraCode-regular.ttf"),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            ),
                            TextSection::new(
                                "Items go here",
                                TextStyle {
                                    font: asset_server
                                        .load("fonts/FiraCode-regular.ttf"),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            ),
                        ]),
                        InventoryUI,
                    ));
                });
        });
}

fn update_inventory(
    player_inventory: Query<(&Inventory, With<Player>)>,
    mut ui_inventory: Query<(&mut Text, With<InventoryUI>)>,
    asset_server: Res<AssetServer>,
) {
    if let Some((inventory, _)) = player_inventory.iter().next() {
        if let Some((mut text, _)) = ui_inventory.iter_mut().next() {
            let items: Vec<TextSection> = inventory
                .hand
                .iter()
                .chain(inventory.belt.iter())
                .chain(inventory.backpack.iter())
                .map(|item| {
                    TextSection::new(
                        item,
                        TextStyle {
                            font: asset_server
                                .load("fonts/FiraCode-Regular.ttf"),
                            font_size: 20.0,
                            color: Color::WHITE,
                        },
                    )
                })
                .intersperse(TextSection::new(
                    "\n",
                    TextStyle {
                        font: asset_server.load("fonts/FiraCode-Regular.ttf"),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                ))
                .collect();
            *text = Text::from_sections(items);
        }
    }
}
