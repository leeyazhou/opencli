#[derive(Debug, Clone)]
pub struct RunArgs {
    pub files: Vec<String>,
    pub dirs: Vec<String>,
    pub prompt: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct A2aArgs {
    pub role: String,
    pub context: Option<String>,
    pub max_steps: Option<usize>,
    pub task: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct A2aBatchArgs {
    pub file: String,
    pub concurrency: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum ConfigCommand {
    Init,
    Show,
    Doctor,
}

#[derive(Debug, Clone)]
pub enum SessionCommand {
    List,
    Resume { id: String },
    Delete { id: String },
    Rename { id: String, title: String },
}

#[derive(Debug, Clone)]
pub struct AuditListArgs {
    pub limit: usize,
    pub tool: Option<String>,
    pub event_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuditTailArgs {
    pub lines: usize,
    pub follow: bool,
    pub tool: Option<String>,
    pub event_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuditExportArgs {
    pub output: String,
    pub tool: Option<String>,
    pub event_type: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AuditCommand {
    List(AuditListArgs),
    Tail(AuditTailArgs),
    Stats,
    Graph,
    Clear,
    Export(AuditExportArgs),
}

#[derive(Debug, Clone)]
pub enum Command {
    Chat,
    #[cfg(feature = "tui")]
    Tui,
    A2a(A2aArgs),
    A2aBatch(A2aBatchArgs),
    Models,
    Run(RunArgs),
    Config(ConfigCommand),
    Session(SessionCommand),
    Audit(AuditCommand),
}
