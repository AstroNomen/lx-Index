#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use std::error::Error;
use std::sync::{Arc, Mutex};

use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use lx_index::i18n::get_i18n_strings;
use lx_index::models::PaperStatus;
use lx_index::ops::{
    copy_bibtex_to_clipboard, copy_citekey_to_clipboard, pick_folder, reveal_in_file_manager,
};
use lx_index::state::AppState;
#[cfg(target_os = "macos")]
use lx_index::vibrancy::{setup_macos_menu_bar, setup_macos_vibrancy};
use lx_index::watcher::{create_shared_watcher, restart_watcher, SharedWatcher};
use lx_index::{AppWindow, SlintCategoryItem, SlintPaperItem, SlintVaultItem};

fn sync_ui(ui: &AppWindow, state: &AppState) {
    // 0. Sync Theme & Localization & Version
    ui.set_app_version(format!("v{}", env!("CARGO_PKG_VERSION")).into());
    ui.set_theme(state.theme.to_slint_theme());
    ui.set_strings(get_i18n_strings(&state.language));
    ui.set_current_theme_id(state.theme.id.clone().into());
    ui.set_current_language(state.language.clone().into());
    ui.set_filter_star_5(state.filter_stars.contains(&5));
    ui.set_filter_star_4(state.filter_stars.contains(&4));
    ui.set_filter_star_3(state.filter_stars.contains(&3));
    ui.set_filter_star_2(state.filter_stars.contains(&2));
    ui.set_filter_star_1(state.filter_stars.contains(&1));
    ui.set_filter_star_0(state.filter_stars.contains(&0));
    ui.set_filter_stars_count(state.filter_stars.len() as i32);

    let summary = if state.filter_stars.is_empty() {
        "".to_string()
    } else {
        state
            .filter_stars
            .iter()
            .map(|s| format!("{}★", s))
            .collect::<Vec<_>>()
            .join(", ")
    };
    ui.set_filter_stars_summary(summary.into());

    #[cfg(target_os = "macos")]
    {
        ui.set_brand_script_font("Snell Roundhand".into());
        ui.set_brand_serif_font("Baskerville".into());
    }
    #[cfg(not(target_os = "macos"))]
    {
        ui.set_brand_script_font("Segoe Script".into());
        ui.set_brand_serif_font("Georgia".into());
    }

    // Settings overlay & tab
    ui.set_show_settings(state.show_settings);
    ui.set_settings_tab(state.settings_tab);

    // Sync Registered Vaults List
    let current_path_str = state.vault_path.as_ref().map(|p| p.display().to_string()).unwrap_or_default();
    let vault_items: Vec<SlintVaultItem> = state
        .registered_vaults
        .iter()
        .map(|v| SlintVaultItem {
            name: v.name.clone().into(),
            path: v.path.clone().into(),
            is_active: v.path == current_path_str,
        })
        .collect();
    ui.set_registered_vaults(ModelRc::new(VecModel::from(vault_items)));

    // 1. Vault Header Info
    if let Some(ref path) = state.vault_path {
        let path_str = path.display().to_string();
        let display_name = state
            .registered_vaults
            .iter()
            .find(|v| v.path == path_str)
            .map(|v| v.name.clone())
            .unwrap_or_else(|| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Vault")
                    .to_string()
            });
        ui.set_vault_name(display_name.into());
        ui.set_vault_path(path_str.into());
    } else {
        let placeholder = if state.language == "zh" {
            "在左下角选择或添加文献库"
        } else {
            "Select or add a library in bottom-left"
        };
        ui.set_vault_name(placeholder.into());
        ui.set_vault_path("".into());
    }
    ui.set_total_count(state.library.items.len() as i32);

    // 2. Perspective Counts (including uncatalogued)
    let (c_all, c_unread, c_prio, c_disc, c_uncat) = state.perspective_counts();
    ui.set_count_all(c_all as i32);
    ui.set_count_unread(c_unread as i32);
    ui.set_count_priority(c_prio as i32);
    ui.set_count_discarded(c_disc as i32);
    ui.set_count_uncatalogued(c_uncat as i32);

    ui.set_active_perspective(state.active_perspective.clone().into());
    ui.set_active_field(state.active_field.clone().unwrap_or_default().into());
    ui.set_active_project(state.active_project.clone().unwrap_or_default().into());
    ui.set_active_tag(state.active_tag.clone().unwrap_or_default().into());
    ui.set_tag_sort_mode(state.tag_sort_mode);

    // 3. Category Lists (Fields, Projects, and Tags)
    let field_items: Vec<SlintCategoryItem> = state
        .field_counts()
        .into_iter()
        .map(|(name, count)| SlintCategoryItem {
            name: name.into(),
            count: count as i32,
        })
        .collect();
    ui.set_field_categories(ModelRc::new(VecModel::from(field_items)));

    let proj_items: Vec<SlintCategoryItem> = state
        .project_counts()
        .into_iter()
        .map(|(name, count)| SlintCategoryItem {
            name: name.into(),
            count: count as i32,
        })
        .collect();
    ui.set_project_categories(ModelRc::new(VecModel::from(proj_items)));

    let tag_items: Vec<SlintCategoryItem> = state
        .tag_counts()
        .into_iter()
        .map(|(name, count)| SlintCategoryItem {
            name: name.into(),
            count: count as i32,
        })
        .collect();
    ui.set_tag_categories(ModelRc::new(VecModel::from(tag_items)));

    // 4. Filtered Items in Main List View
    let filtered = state.filtered_items();
    let slint_items: Vec<SlintPaperItem> = filtered.iter().map(|item| item.to_slint_item()).collect();
    let count = slint_items.len();
    ui.set_items(ModelRc::new(VecModel::from(slint_items)));

    // Update selected index in the filtered view
    let mut selected_idx = -1;
    if let Some(ref sel_id) = state.selected_id {
        if let Some(pos) = filtered.iter().position(|i| &i.id == sel_id) {
            selected_idx = pos as i32;
        }
    }
    ui.set_selected_index(selected_idx);

    // 5. Status text (clean, zero emoji)
    let status_str = if state.language == "zh" {
        if c_uncat > 0 {
            format!("显示 {} 篇文献 • {} 篇待入库", count, c_uncat)
        } else {
            format!("显示 {} 篇文献 • 共 {} 篇", count, state.library.items.len())
        }
    } else {
        if c_uncat > 0 {
            format!("Showing {} items • {} uncatalogued", count, c_uncat)
        } else {
            format!("Showing {} of {} items", count, state.library.items.len())
        }
    };
    ui.set_status_bar_text(status_str.into());

    // 6. Inspector Pane
    sync_inspector(ui, state);
}

fn sync_inspector(ui: &AppWindow, state: &AppState) {
    if let Some(item) = state.selected_item() {
        ui.set_has_selection(true);
        ui.set_inspector_id(item.id.clone().into());
        ui.set_inspector_basename(item.basename.clone().into());
        ui.set_inspector_title(item.metadata.title.clone().into());
        ui.set_inspector_citekey(item.metadata.citekey.clone().unwrap_or_default().into());
        ui.set_inspector_authors(item.metadata.authors.join(", ").into());
        ui.set_inspector_year(item.metadata.year as i32);
        ui.set_inspector_field(item.field.clone().into());
        ui.set_inspector_doi(item.metadata.doi.clone().unwrap_or_default().into());
        ui.set_inspector_status(item.metadata.status.as_code());
        ui.set_inspector_importance(item.metadata.importance as i32);
        ui.set_inspector_summary(item.metadata.summary.clone().unwrap_or_default().into());

        let projects: Vec<SharedString> = item
            .metadata
            .projects
            .iter()
            .map(|p| p.clone().into())
            .collect();
        ui.set_inspector_projects(ModelRc::new(VecModel::from(projects)));

        let tags: Vec<SharedString> = item
            .metadata
            .tags
            .iter()
            .map(|t| t.clone().into())
            .collect();
        ui.set_inspector_tags(ModelRc::new(VecModel::from(tags)));

        ui.set_inspector_has_pdf(item.has_pdf);
        ui.set_inspector_has_note(item.has_note);
        ui.set_inspector_missing_pdf(item.is_missing_pdf);
    } else {
        ui.set_has_selection(false);
        ui.set_inspector_id("".into());
        ui.set_inspector_basename("".into());
        ui.set_inspector_title("".into());
        ui.set_inspector_citekey("".into());
        ui.set_inspector_authors("".into());
        ui.set_inspector_year(0);
        ui.set_inspector_field("".into());
        ui.set_inspector_doi("".into());
        ui.set_inspector_status(0);
        ui.set_inspector_importance(0);
        ui.set_inspector_summary("".into());
        ui.set_inspector_projects(ModelRc::new(VecModel::from(Vec::<SharedString>::new())));
        ui.set_inspector_tags(ModelRc::new(VecModel::from(Vec::<SharedString>::new())));
        ui.set_inspector_has_pdf(false);
        ui.set_inspector_has_note(false);
        ui.set_inspector_missing_pdf(false);
        ui.set_show_tag_suggestions(false);
    }
}

fn setup_watcher(
    watcher: &SharedWatcher,
    vault_path: &std::path::Path,
    ui_handle: slint::Weak<AppWindow>,
    state_rc: Arc<Mutex<AppState>>,
) {
    restart_watcher(watcher, vault_path, move || {
        let ui_handle = ui_handle.clone();
        let state_rc = state_rc.clone();
        let _ = slint::invoke_from_event_loop(move || {
            let mut state = state_rc.lock().unwrap();
            let _ = state.reload_current_vault();
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    });
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let watcher = create_shared_watcher();

    let initial_state = AppState::new();
    let state = Arc::new(Mutex::new(initial_state));

    // Setup initial watcher if vault loaded
    {
        let st = state.lock().unwrap();
        if let Some(ref path) = st.vault_path {
            setup_watcher(&watcher, path, ui.as_weak(), state.clone());
        }
    }

    // 1. Search Query Callback
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_search_query_changed(move |query| {
            let mut state = state_rc.lock().unwrap();
            state.search_query = query.to_string();
            state.selected_id = None;
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }

    // 2. Select Perspective Callback (Toggle support)
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_select_perspective(move |persp| {
            let mut state = state_rc.lock().unwrap();
            let p = persp.to_string();
            if state.active_perspective == p && p != "all" {
                state.active_perspective = "all".to_string();
            } else {
                state.active_perspective = p;
            }
            state.active_field = None;
            state.active_project = None;
            state.selected_id = None;
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }

    // 3. Select Field Callback (Toggle support)
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_select_field(move |field| {
            let mut state = state_rc.lock().unwrap();
            let f = field.to_string();
            if state.active_field.as_deref() == Some(&f) {
                state.active_field = None;
            } else {
                state.active_field = Some(f);
            }
            state.selected_id = None;
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }

    // 4. Select Project Callback (Toggle support)
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_select_project(move |proj| {
            let mut state = state_rc.lock().unwrap();
            let p = proj.to_string();
            if state.active_project.as_deref() == Some(&p) {
                state.active_project = None;
            } else {
                state.active_project = Some(p);
            }
            state.selected_id = None;
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }

    // 5. Clear Filters Callback
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_clear_filters(move || {
            let mut state = state_rc.lock().unwrap();
            state.search_query.clear();
            state.active_perspective = "all".to_string();
            state.active_field = None;
            state.active_project = None;
            state.active_tag = None;
            state.clear_filter_stars();
            state.selected_id = None;
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_search_query("".into());
                #[cfg(target_os = "windows")]
                ui.set_search_expanded(false);
                ui.set_star_filter_open(false);
                ui.set_show_tag_suggestions(false);
                sync_ui(&ui, &state);
            }
        });
    }

    // 5b. Toggle Filter Star Callback
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_toggle_filter_star(move |star| {
            let mut state = state_rc.lock().unwrap();
            state.toggle_filter_star(star as u8);
            state.selected_id = None;
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }

    // 5c. Clear Filter Stars Callback
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_clear_filter_stars(move || {
            let mut state = state_rc.lock().unwrap();
            state.clear_filter_stars();
            state.selected_id = None;
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }

    // 5d. Select All Filter Stars Callback
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_select_all_filter_stars(move || {
            let mut state = state_rc.lock().unwrap();
            state.select_all_filter_stars();
            state.selected_id = None;
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }

    // 6. Select Item Callback (Toggle support & selection cancellation)
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_select_item(move |idx| {
            let mut state = state_rc.lock().unwrap();
            let target_id = if idx >= 0 {
                state.filtered_items().get(idx as usize).map(|i| i.id.clone())
            } else {
                None
            };

            if let Some(ref tid) = target_id {
                if state.selected_id.as_deref() == Some(tid.as_str()) {
                    state.selected_id = None;
                } else {
                    state.selected_id = Some(tid.clone());
                }
            } else {
                state.selected_id = None;
            }

            if let Some(ui) = ui_handle.upgrade() {
                let sel_idx = if let Some(ref sel_id) = state.selected_id {
                    state.filtered_items().iter().position(|i| &i.id == sel_id).map(|p| p as i32).unwrap_or(-1)
                } else {
                    -1
                };
                ui.set_selected_index(sel_idx);
                sync_inspector(&ui, &state);
            }
        });
    }

    // 7. Choose Vault Callback
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        let watcher_clone = watcher.clone();
        ui.on_choose_vault(move || {
            if let Some(dir) = pick_folder() {
                let mut state = state_rc.lock().unwrap();
                if let Err(e) = state.load_vault(&dir) {
                    eprintln!("Failed to load vault: {}", e);
                } else {
                    setup_watcher(&watcher_clone, &dir, ui_handle.clone(), state_rc.clone());
                }
                if let Some(ui) = ui_handle.upgrade() {
                    sync_ui(&ui, &state);
                }
            }
        });
    }

    // 8. Refresh Vault Callback
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_refresh_vault(move || {
            let mut state = state_rc.lock().unwrap();
            let _ = state.reload_current_vault();
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }

    // 9. Ingestion: Import All Uncatalogued Papers
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_import_all_uncatalogued(move || {
            let mut state = state_rc.lock().unwrap();
            match state.import_all_uncatalogued() {
                Ok(count) => {
                    println!("[Ingestion] Successfully imported {} new paper(s)", count);
                }
                Err(e) => {
                    eprintln!("[Ingestion Error] {}", e);
                }
            }
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }

    // 10. Ingestion: Import Selected Paper
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_import_selected(move || {
            let mut state = state_rc.lock().unwrap();
            if let Some(sel_id) = state.selected_id.clone() {
                let _ = state.import_single_paper(&sel_id);
            }
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }

    // 11. Ingestion: Import Specific Item by List Index
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_import_item(move |idx| {
            let mut state = state_rc.lock().unwrap();
            let id = {
                let filtered = state.filtered_items();
                filtered.get(idx as usize).map(|i| i.id.clone())
            };
            if let Some(item_id) = id {
                let _ = state.import_single_paper(&item_id);
            }
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }

    // 12. Settings & Vault Management Callbacks
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_open_settings(move || {
            let mut state = state_rc.lock().unwrap();
            state.show_settings = true;
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_open_settings_tab(move |tab| {
            let mut state = state_rc.lock().unwrap();
            state.show_settings = true;
            state.settings_tab = tab;
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_close_settings(move || {
            let mut state = state_rc.lock().unwrap();
            state.show_settings = false;
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_set_settings_tab(move |tab| {
            let mut state = state_rc.lock().unwrap();
            state.settings_tab = tab;
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_settings_tab(tab);
            }
        });
    }
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_switch_theme(move |theme_id| {
            let mut state = state_rc.lock().unwrap();
            state.set_theme(&theme_id);
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_switch_language(move |lang| {
            let mut state = state_rc.lock().unwrap();
            state.set_language(&lang);
            #[cfg(target_os = "macos")]
            setup_macos_menu_bar(&lang);
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        let watcher_clone = watcher.clone();
        ui.on_add_vault_dialog(move || {
            if let Some(dir) = pick_folder() {
                let mut state = state_rc.lock().unwrap();
                if let Err(e) = state.add_vault(&dir) {
                    eprintln!("Failed to add vault: {}", e);
                } else {
                    setup_watcher(&watcher_clone, &dir, ui_handle.clone(), state_rc.clone());
                }
                if let Some(ui) = ui_handle.upgrade() {
                    sync_ui(&ui, &state);
                }
            }
        });
    }
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_remove_vault(move |path| {
            let mut state = state_rc.lock().unwrap();
            state.remove_vault(&path);
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_save_rename_vault(move |path, new_name| {
            let mut state = state_rc.lock().unwrap();
            state.rename_vault(&path, &new_name);
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        let watcher_clone = watcher.clone();
        ui.on_switch_vault(move |path| {
            let mut state = state_rc.lock().unwrap();
            if let Err(e) = state.switch_vault(&path) {
                eprintln!("Failed to switch vault: {}", e);
            } else if let Some(ref p) = state.vault_path {
                setup_watcher(&watcher_clone, p, ui_handle.clone(), state_rc.clone());
            }
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }

    // 13. Update Status in Inspector
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_update_status(move |code| {
            let mut state = state_rc.lock().unwrap();
            let status = PaperStatus::from_code(code);
            let _ = state.mutate_selected(|meta| meta.status = status);
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }

    // 14. Update Importance in Inspector
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_update_importance(move |stars| {
            let mut state = state_rc.lock().unwrap();
            let _ = state.mutate_selected(|meta| meta.importance = stars as u8);
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }

    // 14b. Update Summary / Appendix in Inspector
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_update_summary(move |summary| {
            let mut state = state_rc.lock().unwrap();
            let s = summary.trim().to_string();
            let _ = state.mutate_selected(|meta| {
                meta.summary = if s.is_empty() { None } else { Some(s) };
            });
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }

    // 15. Add Project Tag
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_add_project(move |proj| {
            let mut state = state_rc.lock().unwrap();
            let p = proj.trim().to_string();
            if !p.is_empty() {
                let _ = state.mutate_selected(|meta| {
                    if !meta.projects.contains(&p) {
                        meta.projects.push(p);
                    }
                });
            }
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }

    // 16. Remove Project Tag
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_remove_project(move |idx| {
            let mut state = state_rc.lock().unwrap();
            let _ = state.mutate_selected(|meta| {
                if (idx as usize) < meta.projects.len() {
                    meta.projects.remove(idx as usize);
                }
            });
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }

    // 17. Add Tag
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_add_tag(move |tag| {
            let mut state = state_rc.lock().unwrap();
            let t = tag.trim().to_string();
            if !t.is_empty() {
                let _ = state.mutate_selected(|meta| {
                    if !meta.tags.contains(&t) {
                        meta.tags.push(t);
                    }
                });
            }
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_show_tag_suggestions(false);
                sync_ui(&ui, &state);
            }
        });
    }

    // 18. Remove Tag
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_remove_tag(move |idx| {
            let mut state = state_rc.lock().unwrap();
            let _ = state.mutate_selected(|meta| {
                if (idx as usize) < meta.tags.len() {
                    meta.tags.remove(idx as usize);
                }
            });
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }

    // 18b. Select / Toggle Tag Filter
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_select_tag(move |tag| {
            let mut state = state_rc.lock().unwrap();
            let trimmed = tag.trim().to_string();
            state.set_active_tag(if trimmed.is_empty() { None } else { Some(trimmed) });
            state.selected_id = None;
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }

    // 18c. Toggle Tag Flyout
    {
        let ui_handle = ui.as_weak();
        ui.on_toggle_tag_flyout(move || {
            if let Some(ui) = ui_handle.upgrade() {
                let cur = ui.get_show_tag_flyout();
                ui.set_show_tag_flyout(!cur);
            }
        });
    }

    // 18d. Set Tag Sort Mode
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_set_tag_sort_mode(move |mode| {
            let mut state = state_rc.lock().unwrap();
            state.tag_sort_mode = mode;
            if let Some(ui) = ui_handle.upgrade() {
                let tag_items: Vec<SlintCategoryItem> = state
                    .tag_counts()
                    .into_iter()
                    .map(|(name, count)| SlintCategoryItem {
                        name: name.into(),
                        count: count as i32,
                    })
                    .collect();
                ui.set_tag_categories(ModelRc::new(VecModel::from(tag_items)));
                ui.set_tag_sort_mode(mode);
            }
        });
    }

    // 18e. Filter Tag Suggestions
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_filter_tag_suggestions(move |prefix| {
            let state = state_rc.lock().unwrap();
            let current_tags = state
                .selected_item()
                .map(|i| i.metadata.tags.clone())
                .unwrap_or_default();
            let suggestions = state.get_tag_suggestions(&prefix, &current_tags, 6);
            if let Some(ui) = ui_handle.upgrade() {
                let has_sugg = !suggestions.is_empty();
                let sugg_models: Vec<slint::SharedString> = suggestions.into_iter().map(Into::into).collect();
                ui.set_tag_suggestions(ModelRc::new(VecModel::from(sugg_models)));
                ui.set_show_tag_suggestions(has_sugg);
            }
        });
    }

    // 18f. Apply Tag Suggestion
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_apply_tag_suggestion(move |sugg| {
            let mut state = state_rc.lock().unwrap();
            let t = sugg.trim().to_string();
            if !t.is_empty() {
                let _ = state.mutate_selected(|meta| {
                    if !meta.tags.contains(&t) {
                        meta.tags.push(t);
                    }
                });
            }
            if let Some(ui) = ui_handle.upgrade() {
                ui.set_new_tag_input("".into());
                ui.set_show_tag_suggestions(false);
                sync_ui(&ui, &state);
            }
        });
    }

    // 19. Reveal PDF
    {
        let state_rc = state.clone();
        ui.on_reveal_pdf(move || {
            let state = state_rc.lock().unwrap();
            if let Some(item) = state.selected_item() {
                if let Some(ref pdf) = item.pdf_path {
                    let _ = reveal_in_file_manager(pdf);
                }
            }
        });
    }

    // 20. Reveal Note
    {
        let state_rc = state.clone();
        ui.on_reveal_note(move || {
            let state = state_rc.lock().unwrap();
            if let Some(item) = state.selected_item() {
                if let Some(ref note) = item.note_path {
                    let _ = reveal_in_file_manager(note);
                }
            }
        });
    }

    // 21. Copy BibTeX Citation
    {
        let state_rc = state.clone();
        ui.on_copy_bibtex(move || {
            let state = state_rc.lock().unwrap();
            if let Some(item) = state.selected_item() {
                let empty_path = std::path::Path::new("");
                let note_path = item.note_path.as_deref().unwrap_or(empty_path);
                let _ = copy_bibtex_to_clipboard(note_path, Some(&item.metadata));
            }
        });
    }

    // 22. Copy Citekey
    {
        let state_rc = state.clone();
        ui.on_copy_citekey(move || {
            let state = state_rc.lock().unwrap();
            if let Some(item) = state.selected_item() {
                if let Some(ref key) = item.metadata.citekey {
                    let _ = copy_citekey_to_clipboard(key);
                }
            }
        });
    }

    // 23. Move Paper to OS Trash
    {
        let ui_handle = ui.as_weak();
        let state_rc = state.clone();
        ui.on_delete_paper(move || {
            let mut state = state_rc.lock().unwrap();
            if let Err(e) = state.delete_selected() {
                eprintln!("Failed to delete paper: {}", e);
            }
            if let Some(ui) = ui_handle.upgrade() {
                sync_ui(&ui, &state);
            }
        });
    }

    // 24. Frameless Window Controls (Windows only)
    #[cfg(target_os = "windows")]
    {
        ui.on_window_drag(move || {
            lx_index::ops::window_drag();
        });
        ui.on_window_minimize(move || {
            lx_index::ops::window_minimize();
        });
        ui.on_window_toggle_maximize(move || {
            lx_index::ops::window_toggle_maximize();
        });
        ui.on_window_close(move || {
            lx_index::ops::window_close();
        });
        ui.set_window_no_frame(true);
    }

    // Initialize UI State
    {
        let st = state.lock().unwrap();
        sync_ui(&ui, &st);
    }

    // Show window first so native NSWindow exists on macOS
    ui.show()?;

    // Setup native Windows borderless drop shadow & DWM styling
    #[cfg(target_os = "windows")]
    let _win_shadow_timer = {
        let t = slint::Timer::default();
        t.start(
            slint::TimerMode::SingleShot,
            std::time::Duration::from_millis(60),
            move || {
                lx_index::ops::setup_windows_window();
            },
        );
        t
    };

    // Setup native macOS seamless titlebar integration, system menu bar & window sizing bounds
    #[cfg(target_os = "macos")]
    let (_timer_fast, _timer_fallback) = {
        let current_lang = {
            let st = state.lock().unwrap();
            st.language.clone()
        };
        let lang1 = current_lang.clone();
        let lang2 = current_lang;
        let t1 = slint::Timer::default();
        t1.start(
            slint::TimerMode::SingleShot,
            std::time::Duration::from_millis(50),
            move || {
                let _ = setup_macos_vibrancy();
                let _ = setup_macos_menu_bar(&lang1);
            },
        );
        let t2 = slint::Timer::default();
        t2.start(
            slint::TimerMode::SingleShot,
            std::time::Duration::from_millis(200),
            move || {
                let _ = setup_macos_vibrancy();
                let _ = setup_macos_menu_bar(&lang2);
            },
        );
        (t1, t2)
    };

    slint::run_event_loop()?;
    Ok(())
}
