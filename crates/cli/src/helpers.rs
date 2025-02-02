use console::{set_colors_enabled, set_colors_enabled_stderr};
use indicatif::{ProgressBar, ProgressStyle};
use moon_launchpad::check_version;
use moon_logger::debug;
use moon_terminal::{create_theme, get_checkpoint_prefix, Checkpoint};
use moon_utils::is_test_env;
use starbase_styles::color::{self, no_color, supports_color};
use std::env;
use std::time::Duration;

pub fn create_progress_bar<S: AsRef<str>, F: AsRef<str>>(start: S) -> impl FnOnce(F, bool) {
    let pb = ProgressBar::new_spinner();
    pb.set_message(start.as_ref().to_owned());
    pb.enable_steady_tick(Duration::from_millis(50));

    move |finish, passed| {
        let theme = create_theme();

        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{prefix} {msg}")
                .unwrap(),
        );

        if passed {
            pb.set_prefix(theme.success_prefix.to_string());
        } else {
            pb.set_prefix(theme.error_prefix.to_string());
        }

        pb.finish_with_message(finish.as_ref().to_owned());
    }
}

fn setup_no_colors() {
    env::set_var("NO_COLOR", "1");
    // https://github.com/mitsuhiko/clicolors-control/issues/19
    env::set_var("CLICOLOR", "0");

    set_colors_enabled(false);
    set_colors_enabled_stderr(false);
}

pub fn setup_colors(force: bool) {
    // If being forced by --color or other env vars
    if force
        || env::var("MOON_COLOR").is_ok()
        || env::var("FORCE_COLOR").is_ok()
        || env::var("CLICOLOR_FORCE").is_ok()
    {
        let mut color_level = env::var("MOON_COLOR")
            .or_else(|_| env::var("FORCE_COLOR"))
            .unwrap_or("3".to_owned());

        // https://nodejs.org/api/cli.html#force_color1-2-3
        if color_level.is_empty() || color_level == "true" {
            color_level = "1".to_owned();
        } else if color_level == "false" {
            color_level = "0".to_owned();
        }

        if color_level == "0" {
            setup_no_colors();
        } else {
            set_colors_enabled(true);
            set_colors_enabled_stderr(true);

            // https://bixense.com/clicolors/
            env::set_var("CLICOLOR_FORCE", &color_level);
            env::set_var("FORCE_COLOR", &color_level);
        }

        return;
    }

    if no_color() {
        setup_no_colors();
    } else {
        env::set_var("CLICOLOR", supports_color().to_string());
    }
}

pub async fn check_for_new_version() {
    if !is_test_env() && moon::is_telemetry_enabled() {
        let version = env!("CARGO_PKG_VERSION");
        let prefix = get_checkpoint_prefix(Checkpoint::Announcement);

        match check_version(version, false).await {
            Ok(Some(newer_version)) => {
                println!(
                    "{} There's a new version of moon available, {}!",
                    prefix,
                    color::success(newer_version.current_version)
                );

                if let Some(newer_message) = newer_version.message {
                    println!("{} {}", prefix, newer_message);
                }

                println!(
                    "{} Run {} or install from {}",
                    prefix,
                    color::success("moon upgrade"),
                    color::url("https://moonrepo.dev/docs/install"),
                );
            }
            Err(error) => {
                debug!(target: "moon:cli", "Failed to check for current version: {}", error);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serial_test::serial;

    fn reset_vars() {
        env::remove_var("NO_COLOR");
        env::remove_var("CLICOLOR");
        env::remove_var("CLICOLOR_FORCE");
        env::remove_var("FORCE_COLOR");
        env::remove_var("MOON_COLOR");
    }

    mod setup_color {
        use super::*;

        mod no_color {
            use super::*;

            #[test]
            #[serial]
            fn sets_vars() {
                env::set_var("NO_COLOR", "1");

                setup_colors(false);

                assert_eq!(env::var("CLICOLOR").unwrap(), "0");
                assert_eq!(env::var("NO_COLOR").unwrap(), "1");

                reset_vars();
            }
        }

        mod forced_color {
            use super::*;

            // #[test]
            // #[serial]
            // fn forces_via_arg() {
            //     setup_colors(true);

            //     assert_eq!(env::var("CLICOLOR_FORCE").unwrap(), "2");
            //     assert_eq!(env::var("FORCE_COLOR").unwrap(), "2");
            //     assert!(env::var("NO_COLOR").is_err());

            //     reset_vars();
            // }

            // #[test]
            // #[serial]
            // fn forces_over_no_color() {
            //     env::set_var("NO_COLOR", "1");

            //     setup_colors(true);

            //     assert_eq!(env::var("CLICOLOR_FORCE").unwrap(), "2");
            //     assert_eq!(env::var("FORCE_COLOR").unwrap(), "2");
            //     assert_eq!(env::var("NO_COLOR").unwrap(), "1");

            //     reset_vars();
            // }

            #[test]
            #[serial]
            fn disables_if_zero() {
                for var in ["MOON_COLOR", "FORCE_COLOR"] {
                    env::set_var(var, "0");

                    setup_colors(false);

                    assert_eq!(env::var("CLICOLOR").unwrap(), "0");
                    assert_eq!(env::var("NO_COLOR").unwrap(), "1");

                    reset_vars();
                }
            }

            #[test]
            #[serial]
            fn disables_if_false_string() {
                for var in ["MOON_COLOR", "FORCE_COLOR"] {
                    env::set_var(var, "false");

                    setup_colors(false);

                    assert_eq!(env::var("CLICOLOR").unwrap(), "0");
                    assert_eq!(env::var("NO_COLOR").unwrap(), "1");

                    reset_vars();
                }
            }

            #[test]
            #[serial]
            fn enables_if_empty_string() {
                for var in ["MOON_COLOR", "FORCE_COLOR"] {
                    env::set_var(var, "");

                    setup_colors(false);

                    assert_eq!(env::var("CLICOLOR_FORCE").unwrap(), "1");
                    assert_eq!(env::var("FORCE_COLOR").unwrap(), "1");

                    reset_vars();
                }
            }

            #[test]
            #[serial]
            fn enables_if_true_string() {
                for var in ["MOON_COLOR", "FORCE_COLOR"] {
                    env::set_var(var, "true");

                    setup_colors(false);

                    assert_eq!(env::var("CLICOLOR_FORCE").unwrap(), "1");
                    assert_eq!(env::var("FORCE_COLOR").unwrap(), "1");

                    reset_vars();
                }
            }
        }
    }
}
