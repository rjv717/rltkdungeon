use rltk::{RGB, Rltk, VirtualKeyCode};

use crate::state_machine::{State, RunState};
use crate::systems::saveload_system::does_save_exist;

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    DebugMap,
    DebugMapCont,
    Quit,
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection { selected: MainMenuSelection },
    Selected { selected: MainMenuSelection },
}

pub fn main_menu(gs : &mut State, ctx : &mut Rltk) -> MainMenuResult {
    let save_exists = does_save_exist();
    let runstate = gs.ecs.fetch::<RunState>();

    ctx.print_color_centered(15, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), "Rust Roguelike Tutorial");

    if let RunState::MainMenu{ menu_selection : selection } = *runstate {
        if selection == MainMenuSelection::NewGame {
            ctx.print_color_centered(24, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Begin New Game");
        } else {
            ctx.print_color_centered(24, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Begin New Game");
        }

        if selection == MainMenuSelection::LoadGame {
            ctx.print_color_centered(25, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Load Game");
        } else {
            ctx.print_color_centered(25, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Load Game");
        }

        if selection == MainMenuSelection::DebugMap {
            ctx.print_color_centered(26, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Debug Map");
        } else {
            ctx.print_color_centered(26, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Debug Map");
        }

        if selection == MainMenuSelection::Quit {
            ctx.print_color_centered(27, RGB::named(rltk::MAGENTA), RGB::named(rltk::BLACK), "Quit");
        } else {
            ctx.print_color_centered(27, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK), "Quit");
        }

        match ctx.key {
            None => return MainMenuResult::NoSelection{ selected: selection },
            Some(key) => {
                match key {
                    // TODO: Fix Menu Selection issues.
                    VirtualKeyCode::Escape => { return MainMenuResult::NoSelection{ selected: MainMenuSelection::Quit } }
                    VirtualKeyCode::Up => {
                        let mut newselection;
                        match selection {
                            MainMenuSelection::NewGame => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::LoadGame => newselection = MainMenuSelection::NewGame,
                            MainMenuSelection::DebugMap => newselection = MainMenuSelection::LoadGame,
                            MainMenuSelection::Quit => newselection = MainMenuSelection::DebugMap,
                            MainMenuSelection::DebugMapCont => newselection = MainMenuSelection::DebugMapCont,
                        }
                        if newselection == MainMenuSelection::LoadGame && !save_exists {
                            newselection = MainMenuSelection::NewGame;
                        }
                        return MainMenuResult::NoSelection{ selected: newselection }
                    }
                    VirtualKeyCode::Down => {
                        let mut newselection;
                        match selection {
                            MainMenuSelection::NewGame => newselection = MainMenuSelection::LoadGame,
                            MainMenuSelection::LoadGame => newselection = MainMenuSelection::DebugMap,
                            MainMenuSelection::DebugMap => newselection = MainMenuSelection::Quit,
                            MainMenuSelection::Quit => newselection = MainMenuSelection::NewGame,
                            MainMenuSelection::DebugMapCont => newselection = MainMenuSelection::DebugMapCont,
                        }
                        if newselection == MainMenuSelection::LoadGame && !save_exists {
                            newselection = MainMenuSelection::DebugMap;
                        }
                        return MainMenuResult::NoSelection{ selected: newselection }
                    }
                    VirtualKeyCode::Return => return MainMenuResult::Selected{ selected : selection },
                    _ => return MainMenuResult::NoSelection{ selected: selection }
                }
            }
        }
    }

    MainMenuResult::NoSelection { selected: MainMenuSelection::NewGame }
}