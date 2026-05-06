#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OobePage {
    Welcome,
    Connect,
    Timezone,
    Privacy,
    Account,
    Ready,
}
