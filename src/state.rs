
#[derive(Default, PartialEq, Eq)]
pub enum AppState {
    #[default]
    Loading,
    IssueToken {
        userid: String,
        password: String,
        tenantid: String,
        identity_url: String,
    },
    Server,
    Quit,
}
