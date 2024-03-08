// Update system;
use self_update;
use self_update::cargo_crate_version;
use self_update::version::bump_is_greater;
use tinyfiledialogs::{MessageBoxIcon, OkCancel};

const OWNER: &str = "Delfi1";
const REPO: &str = "SPE";

pub(super) fn update() -> Result<(), Box<dyn std::error::Error>> {
    let build = self_update::backends::github::Update::configure()
        .repo_owner(OWNER)
        .repo_name(REPO)
        .bin_name("github")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?;

    let latest_ver = build.get_latest_release().unwrap().version;
    if bump_is_greater(build.current_version().as_str(), latest_ver.as_str())? {
        let msg = tinyfiledialogs::message_box_ok_cancel(
            "Updater",
            "New version of SPE was found; \nDo you want to update?",
            MessageBoxIcon::Info,
            OkCancel::Cancel
        );

        if msg == OkCancel::Ok {
            let status = build.update()?;

            println!("Update status: `{}`!", status.version());
        };
    }

    Ok(())
}
