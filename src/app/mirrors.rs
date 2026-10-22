use std::process::Command;

use super::{AppState, CustomRepo, PopupKind, RepoSignOption, RepoSignature, Screen};

#[derive(Debug)]
pub enum MirrorsLoadError {
    ListCountries,
}

impl std::fmt::Display for MirrorsLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MirrorsLoadError::ListCountries => write!(f, "failed to list countries via reflector"),
        }
    }
}

impl std::error::Error for MirrorsLoadError {}

impl AppState {
    pub fn load_mirrors_options(&mut self) -> Result<(), MirrorsLoadError> {
        if self.mirrors_loaded {
            return Ok(());
        }
        if let Ok(output) = Command::new("reflector").arg("--list-countries").output()
            && output.status.success()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut items: Vec<String> = text
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            items.sort();
            self.mirrors_regions_options = items;
        }

        // Fallback small list when reflector is unavailable (Windows/dry-run)
        if (cfg!(windows) || self.dry_run) && self.mirrors_regions_options.is_empty() {
            self.mirrors_regions_options = vec![
                "United States          US   186".into(),
                "Germany                DE   120".into(),
                "France                 FR    80".into(),
                "United Kingdom         GB    75".into(),
            ];
        }

        self.mirrors_loaded = true;
        Ok(())
    }

    pub fn open_mirrors_regions_popup(&mut self) {
        if self.current_screen() != Screen::MirrorsRepos || self.focus != crate::core::types::Focus::Content {
            return;
        }
        let _ = self.load_mirrors_options();
        self.popup_kind = Some(PopupKind::MirrorsRegions);
        self.popup_items = self.mirrors_regions_options.clone();
        self.popup_search_query.clear();
        self.popup_in_search = false;
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_open = true;
    }

    pub fn open_optional_repos_popup(&mut self) {
        if self.current_screen() != Screen::MirrorsRepos || self.focus != crate::core::types::Focus::Content {
            return;
        }
        self.popup_kind = Some(PopupKind::OptionalRepos);
        self.popup_items = self.optional_repos_options.clone();
        self.popup_search_query.clear();
        self.popup_in_search = false;
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = 0;
        self.popup_open = true;
    }

    pub fn open_mirrors_custom_server_input(&mut self) {
        if self.current_screen() != Screen::MirrorsRepos || self.focus != crate::core::types::Focus::Content {
            return;
        }
        self.popup_kind = Some(PopupKind::MirrorsCustomServerInput);
        self.custom_input_buffer.clear();
        self.popup_open = true;
    }

    pub fn open_mirrors_custom_repo_flow(&mut self) {
        if self.current_screen() != Screen::MirrorsRepos || self.focus != crate::core::types::Focus::Content {
            return;
        }
        self.draft_repo_name.clear();
        self.draft_repo_url.clear();
        self.draft_repo_sig_index = 2;
        self.custom_input_buffer.clear();
        self.popup_kind = Some(PopupKind::MirrorsCustomRepoName);
        self.popup_open = true;
    }

    pub fn open_mirrors_custom_repo_url(&mut self) {
        self.custom_input_buffer.clear();
        self.popup_kind = Some(PopupKind::MirrorsCustomRepoUrl);
    }

    pub fn open_mirrors_custom_repo_sig(&mut self) {
        self.popup_kind = Some(PopupKind::MirrorsCustomRepoSig);
        self.popup_items = vec!["Never".into(), "Optional".into(), "Required".into()];
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = self
            .draft_repo_sig_index
            .min(self.popup_items.len().saturating_sub(1));
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn open_mirrors_custom_repo_signopt(&mut self) {
        self.popup_kind = Some(PopupKind::MirrorsCustomRepoSignOpt);
        self.popup_items = vec!["TrustedOnly".into(), "TrustedAll".into()];
        self.popup_visible_indices = (0..self.popup_items.len()).collect();
        self.popup_selected_visible = self
            .draft_repo_signopt_index
            .min(self.popup_items.len().saturating_sub(1));
        self.popup_in_search = false;
        self.popup_search_query.clear();
    }

    pub fn finalize_custom_repo(&mut self) {
        let sig = match self.draft_repo_sig_index {
            0 => RepoSignature::Never,
            1 => RepoSignature::Optional,
            _ => RepoSignature::Required,
        };
        let sign_option = if self.draft_repo_sig_index == 0 {
            None
        } else {
            Some(match self.draft_repo_signopt_index {
                0 => RepoSignOption::TrustedOnly,
                _ => RepoSignOption::TrustedAll,
            })
        };
        self.custom_repos.push(CustomRepo {
            name: self.draft_repo_name.clone(),
            url: self.draft_repo_url.clone(),
            signature: sig,
            sign_option,
        });
        self.close_popup();
    }
}

