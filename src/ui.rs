use crate::{
    files::{
        check_dir_changed, check_open_file_changed, file_dir_path, list_approved_dir_files, CurrentGltfEntity, Directory, EditFileName, FileList, ModelInfo, OpenFile, ShowEditFileName, SortMode
    },
    style::styled_button,
};
use bevy::{
    camera::{Viewport, visibility::RenderLayers},
    light::CascadeShadowConfigBuilder,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, poll_once},
    window::PrimaryWindow,
};
use bevy_egui::{
    EguiContext, EguiContexts, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass,
    PrimaryEguiContext, egui,
};
use bevy_enhanced_input::condition::press::Press;
use bevy_enhanced_input::{action::Action, actions, prelude::*};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bytesize::ByteSize;
use std::{f32::consts::PI, fs, path::Path};

#[derive(Component)]
pub struct UiKeyAction;
#[derive(InputAction)]
#[action_output(bool)]
pub struct FileNavUp;

#[derive(InputAction)]
#[action_output(bool)]
pub struct FileNavDown;

pub fn setup_ui(
    mut commands: Commands,
    //mut directory: ResMut<Directory>,
    //mut egui_global_settings: ResMut<EguiGlobalSettings>,
    //mut meshes: ResMut<Assets<Mesh>>,
    //mut materials: ResMut<Assets<StandardMaterial>>,
    //mut sort_mode: ResMut<SortMode>,
) {
    commands.spawn((
        UiKeyAction,
        actions!(UiKeyAction[
             (
                Action::<FileNavUp>::new(),
                Press::new(1.0),
                bindings![KeyCode::ArrowUp, GamepadButton::LeftTrigger],
            ),
             (
                Action::<FileNavDown>::new(),
                Press::new(1.0),
                bindings![KeyCode::ArrowDown, GamepadButton::RightTrigger],
            )
        ]),
    ));
}

#[derive(PartialEq)]
pub enum ViewMode {
    Model,
    Grid,
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::Model
    }
}

#[derive(Default)]
pub struct MyState {
    pub dropped_files: Vec<egui::DroppedFile>,
    pub picked_path: Option<String>,
    pub view_mode: ViewMode,
}

pub type DialogResponse = Option<rfd::FileHandle>;

// then check for keyboard nav stuff
pub fn handle_file_nav_up(
    _trigger: On<Fire<FileNavUp>>,
    mut file_list: ResMut<FileList>,
    mut open_file: ResMut<OpenFile>,
    mut directory: ResMut<Directory>,
) {
    let path = Path::new(&open_file.0)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    
    if let Some(mut index) = file_list.0.iter().position(|x| x.name == path) {
        if index == 0 {
            index = file_list.0.len() - 1;
        } else {
            index -= 1;
        }
        open_file.0 = file_dir_path(directory.0.clone(), file_list.0[index].name.clone());
    }
}

pub fn handle_file_nav_down(
    _trigger: On<Fire<FileNavDown>>,
    mut file_list: ResMut<FileList>,
    mut open_file: ResMut<OpenFile>,
    mut directory: ResMut<Directory>,
) {
    let path = Path::new(&open_file.0)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
   
    if let Some(mut index) = file_list.0.iter().position(|x| x.name == path) {
        index += 1;
        if index + 1 > file_list.0.len() {
            index = 0;
        }
        open_file.0 = file_dir_path(directory.0.clone(), file_list.0[index].name.clone());
    }
}

// This function runs every frame. Therefore, updating the viewport after drawing the gui.
// With a resource which stores the dimensions of the panels, the update of the Viewport can
// be done in another system.
pub fn ui_system(
    mut directory: ResMut<Directory>,
    mut open_file: ResMut<OpenFile>,
    mut contexts: EguiContexts,
    mut camera: Single<&mut Camera, Without<EguiContext>>,
    mut state: Local<MyState>,
    mut file_dialog: Local<Option<Task<DialogResponse>>>,
    window: Single<&mut Window, With<PrimaryWindow>>,
    mut file_list: ResMut<FileList>,
    mut sort_mode: ResMut<SortMode>,
    mut show_edit_file_name: ResMut<ShowEditFileName>,
    mut edit_file_name: ResMut<EditFileName>,
    mut model_info: ResMut<ModelInfo>,
) -> Result {
    // Poll the file dialog task FIRST, before any early returns
    if let Some(file_response) = file_dialog
        .as_mut()
        .and_then(|task| block_on(poll_once(task)))
    {
        state.picked_path = file_response.map(|path| path.path().display().to_string());
        *file_dialog = None;
    }

    let ctx = contexts.ctx_mut()?;

    let mut left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            // text input section
            ui.horizontal(|ui| {
                ui.label("Directory:");
                ui.text_edit_singleline(&mut directory.0);
            });
            ui.label(format!("Open File {}", open_file.0));
            ui.separator();
            ui.label(format!("Browsing {}", directory.0));

            ui.label("Drag-and-drop files onto the window!");

            if ui.button("Open file…").clicked() {
                *file_dialog = Some(
                    AsyncComputeTaskPool::get().spawn(rfd::AsyncFileDialog::new().pick_file()),
                );
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Up").clicked() {
                    let path = std::fs::canonicalize(&directory.0)
                        .unwrap_or_else(|_| std::path::PathBuf::from(&directory.0));
                    if let Some(parent) = path.parent() {
                        directory.0 = parent.to_string_lossy().to_string();
                    } else {
                        warn!("Cannot navigate up from directory: {}", directory.0);
                    }
                }
                if ui.button("Refresh").clicked() {
                    file_list.0 = list_approved_dir_files(&directory.0, *sort_mode);
                }
            });

            if let Some(picked_path) = &state.picked_path {
                ui.horizontal(|ui| {
                    ui.label("Picked file:");
                    ui.monospace(picked_path);
                });
            }

            // Show dropped files (if any):
            if !state.dropped_files.is_empty() {
                ui.group(|ui| {
                    ui.label("Dropped files:");

                    for file in &state.dropped_files {
                        let mut info = if let Some(path) = &file.path {
                            path.display().to_string()
                        } else if !file.name.is_empty() {
                            file.name.clone()
                        } else {
                            "???".to_owned()
                        };

                        let mut additional_info = vec![];
                        if !file.mime.is_empty() {
                            additional_info.push(format!("type: {}", file.mime));
                        }
                        if let Some(bytes) = &file.bytes {
                            additional_info.push(format!("{} bytes", bytes.len()));
                        }
                        if !additional_info.is_empty() {
                            info += &format!(" ({})", additional_info.join(", "));
                        }

                        ui.label(info);
                    }
                });
            }

            ui.separator();

            ui.horizontal(|ui| {
                if styled_button(ui, "Name", false, *sort_mode == SortMode::Name, None).clicked() {
                    *sort_mode = SortMode::Name;
                }
                if styled_button(ui, "Size", false, *sort_mode == SortMode::Size, None).clicked() {
                    *sort_mode = SortMode::Size;
                }
                if styled_button(ui, "Date", false, *sort_mode == SortMode::Date, None).clicked() {
                    *sort_mode = SortMode::Date;
                }
                // if ui.button("Name").clicked() {
                //     *sort_mode = SortMode::Name;
                // }
                // if ui.button("Size").clicked() {
                //     *sort_mode = SortMode::Size;
                // }
                // if ui.button("Date").clicked() {
                //     *sort_mode = SortMode::Date;
                // }
            });
            egui::ScrollArea::vertical().show(ui, |ui| {
                for entry in &file_list.0 {
                    //    ui.label(entry);
                    // if ui.button(format!("{}", filename)).clicked() {
                    //     let path = std::path::Path::new(&directory.0).join(filename);
                    //     if path.is_dir() {
                    //         directory.0 = path.to_str().unwrap_or(&directory.0).to_string();
                    //     } else {
                    //         open_file.0 = path.to_str().unwrap_or("").to_string();
                    //     }
                    //     // let md = std::fs::metadata(filename)
                    //     // if std::fs::
                    //     // // Handle the button click
                    //     println!("You clicked: {} ", filename,);
                    //     // For example, you could trigger opening, previewing, etc.
                    // }
                    let path = std::path::Path::new(&directory.0).join(entry.name.clone());
                    let is_selected = open_file.0 == path.to_str().unwrap_or("").to_string();

                    let response = styled_button(
                        ui,
                        format!("{}", entry.name).as_ref(),
                        path.is_dir(),
                        is_selected,
                        Some(egui::vec2(200.0, 30.0))
                    );

                    // Handle click
                    if response.clicked() {
                        if path.is_dir() {
                            directory.0 = path.to_str().unwrap_or(&directory.0).to_string();
                        } else {
                            open_file.0 = path.to_str().unwrap_or("").to_string();
                        }
                    }
                }
            });

            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width(); // height is ignored, as the panel has a hight of 100% of the screen

    // Collect dropped files:
    // ctx.input(|i| {
    //     if !i.raw.dropped_files.is_empty() {
    //         state.dropped_files.clone_from(&i.raw.dropped_files);
    //     }
    // });

    // ctx.input(|i| {
    //     if i.raw.modifiers.ctrl {
    //         info!("ctrl pressed");
    //     }
    // });

    let mut right = egui::SidePanel::right("right_panel")
        .resizable(true)
        .show(ctx, |ui| {
            if open_file.0 != "".to_string() {
                ui.label("Info");
                match std::fs::metadata(open_file.0.clone()) {
                    Ok(md) => {
                        let m = format!("Size {:?} bytes", ByteSize(md.len()));
                        ui.label(m);
                    }
                    Err(_) => {}
                }

                if ui.button("Delete File").clicked() {
                    match fs::remove_file(open_file.0.clone()) {
                        Ok(_) => {
                            println!("Successfully deleted {:?}", open_file.0);
                            open_file.0 = "".to_string();
                            file_list.0 = list_approved_dir_files(&directory.0, *sort_mode);
                        }
                        Err(e) => println!("Error deleting {:?}\n{:?}", open_file.0, e),
                    }
                }
                ui.separator();
                ui.label(format!("Polygons: {:} ", model_info.polygon_count));
                ui.label(format!("Vertices: {:} ", model_info.vertex_count));
            }
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width(); // height is ignored, as the panel has a height of 100% of the screen

    let mut top = egui::TopBottomPanel::top("top_panel")
        .resizable(true)
        .show(ctx, |ui| {
            // Toggle button for view mode
            ui.horizontal(|ui| {
                let toggle_label = match state.view_mode {
                    ViewMode::Model => "Show Grid",
                    ViewMode::Grid => "Show 3D",
                };
                if ui.button(toggle_label).clicked() {
                    state.view_mode = if state.view_mode == ViewMode::Model {
                        ViewMode::Grid
                    } else {
                        ViewMode::Model
                    };
                }
            });

            let path = open_file.0.clone(); // std::path::Path::new(&directory.0).join(entry.name.clone());
            if path != "".to_string() {
                ui.horizontal(|ui| {
                    if show_edit_file_name.0 {
                        ui.add_sized(ui.available_size()- bevy_egui::egui::Vec2::new(40.0, 0.0), egui::TextEdit::singleline(&mut edit_file_name.0));
                        //ui.text_edit_singleline(&mut edit_file_name.0);
                        if ui.button("Save").clicked() {
                            let src = path.clone();
                            let dest = edit_file_name.0.clone();
                            match fs::rename(src, dest.clone()) {
                                Ok(_) => {
                                    open_file.0 = dest;
                                    show_edit_file_name.0 = false;
                                    file_list.0 = list_approved_dir_files(&directory.0, *sort_mode);
                                }
                                Err(e) => {
                                    //TODO handle this
                                    println!("{:?}", e);
                                }
                            };
                        }
                        if ui.button("Cancel").clicked() {
                            edit_file_name.0 = path;
                            show_edit_file_name.0 = false;
                        }
                    } else {
                        ui.label(open_file.0.to_string());
                        if ui.button("Rename").clicked() {
                            edit_file_name.0 = path;
                            show_edit_file_name.0 = true;
                        }
                    }

                    //let response = styled_button(ui, format!("Rename").as_ref(), false, is_selected);

                    // Handle click
                    // if response.clicked() {
                    //     edit_file_name.0 = path;
                    //     show_edit_file_name.0 = !show_edit_file_name.0;
                    // }
                });
            }
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height(); // width is ignored, as the panel has a width of 100% of the screen
    let mut bottom = egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("");
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height(); // width is ignored, as the panel has a width of 100% of the screen

    // Scale from logical units to physical units.
    left *= window.scale_factor();
    right *= window.scale_factor();
    top *= window.scale_factor();
    bottom *= window.scale_factor();

    // Center view area
    // Render grid of 2D cards if in grid mode, otherwise set camera viewport as usual
    if state.view_mode == ViewMode::Grid {
        egui::CentralPanel::default().show(ctx, |ui| {
             egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("File Grid");
            let num_columns = 4;
            let card_size = egui::vec2(120.0, 120.0);
            let spacing = 16.0;

            egui::Grid::new("file_grid")
                .num_columns(num_columns)
                .spacing([spacing, spacing])
                .show(ui, |ui| {
                    for (i, entry) in file_list.0.iter().enumerate() {
                        ui.vertical_centered(|ui| {
                            // Placeholder for image: use a colored rect for now
                            ui.add_sized(card_size, egui::widgets::ImageButton::new(
                                egui::include_image!("../assets/icons/file.png")
                                
                            ));
                            ui.label(&entry.name);
                        });
                        if (i + 1) % num_columns == 0 {
                            ui.end_row();
                        }
                    }
                });
        });
    });
    } else {
        // -------------------------------------------------
        // |  left   |            top   ^^^^^^   |  right  |
        // |  panel  |           panel  height   |  panel  |
        // |         |                  vvvvvv   |         |
        // |         |---------------------------|         |
        // |         |                           |         |
        // |<-width->|          viewport         |<-width->|
        // |         |                           |         |
        // |         |---------------------------|         |
        // |         |          bottom   ^^^^^^  |         |
        // |         |          panel    height  |         |
        // |         |                   vvvvvv  |         |
        // -------------------------------------------------
        //
        // The upper left point of the viewport is the width of the left panel and the height of the
        // top panel
        //
        // The width of the viewport the width of the top/bottom panel
        // Alternative the width can be calculated as follow:
        // size.x = window width - left panel width - right panel width
        //
        // The height of the viewport is:
        // size.y = window height - top panel height - bottom panel height
        //
        // Therefore we use the alternative for the width, as we can callculate the Viewport as
        // following:

        let pos = UVec2::new(left as u32, top as u32);
        let size = UVec2::new(window.physical_width(), window.physical_height())
            - pos
            - UVec2::new(right as u32, bottom as u32);

        camera.viewport = Some(Viewport {
            physical_position: pos,
            physical_size: size,
            ..default()
        });
    }

    Ok(())
}
